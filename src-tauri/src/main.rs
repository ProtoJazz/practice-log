use chrono::{Local, NaiveDateTime, TimeDelta, Utc};
use dotenv::dotenv;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde_json::json;
use sqlx::{Error, SqlitePool};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::State;
use tauri::{command, Emitter};
use tokio::task;

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
struct PracticePiece {
    id: Option<i64>,        // Unique ID for the piece
    name: String,           // Each piece has a name, which could be NULL
    logs: Vec<PracticeLog>, // Vector of associated practice logs
}
#[derive(Debug, sqlx::FromRow)]
struct RegimenPieceLog {
    regimen_id: i64,
    regimen_date: NaiveDateTime,
    piece_id: Option<i64>,
    piece_name: Option<String>,
    log_id: Option<i64>,
    log_bpm: Option<i64>,
    log_timestamp: Option<NaiveDateTime>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
struct PracticeLog {
    id: Option<i64>,          // Unique ID for the log
    practice_piece_id: i64,   // Foreign key to `practice_piece`,
    bpm: i64,                 // The BPM value at the time of the log
    timestamp: NaiveDateTime, // The timestamp of the log
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
struct PracticeRegiment {
    id: Option<i64>,
    date: NaiveDateTime,        // Unique ID for the regiment
    pieces: Vec<PracticePiece>, // Vector of associated practice pieces
}

#[tauri::command]
async fn create_full_regiment(
    pool: tauri::State<'_, SqlitePool>,
    regiment: PracticeRegiment, // Accept regiment data from the frontend
) -> Result<(), String> {
    // Insert the regiment and pieces into the database
    let date = regiment.date;
    let piece_names: Vec<String> = regiment
        .pieces
        .into_iter()
        .map(|piece| piece.name)
        .collect();

    match insert_practice_regiment_with_transaction(&pool, date, piece_names).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let error_message = format!("Failed to insert regiment in command: {:?}", e);
            println!("{}", error_message); // Log the error
            Err(error_message) // Return the error to the frontend
        }
    }
}

#[command]
async fn mark_active_piece(
    active_piece: State<'_, Arc<Mutex<Option<i64>>>>, // Access the in-memory active piece state
    practice_piece_id: i64, // Accept the ID of the piece to mark as active
) -> Result<(), String> {
    let mut active = active_piece
        .lock()
        .map_err(|e| format!("Failed to lock mutex: {:?}", e))?;
    *active = Some(practice_piece_id); // Set the active piece ID in memory

    println!("Marked practice piece {} as active", practice_piece_id); // Log for debugging
    Ok(())
}

#[command]
async fn get_active_piece(
    active_piece: State<'_, Arc<Mutex<Option<i64>>>>, // Access the in-memory active piece state
) -> Result<Option<i64>, String> {
    let active = active_piece
        .lock()
        .map_err(|e| format!("Failed to lock mutex: {:?}", e))?;
    Ok(*active) // Return the currently active practice piece ID (if any)
}
#[command]
async fn load_practice_regiments_2(pool: tauri::State<'_, SqlitePool>) -> Result<String, String> {
    let results: Vec<RegimenPieceLog> = sqlx::query_as(
        r#"
        SELECT
            r.id as regimen_id, r.date as regimen_date,
            p.id as piece_id, p.name as piece_name,
            l.id as log_id, l.bpm as log_bpm, l.timestamp as log_timestamp
        FROM
            practice_regiment r
        LEFT JOIN
            practice_piece p ON r.id = p.practice_regiment_id
        LEFT JOIN
            practice_piece_log_entry l ON p.id = l.practice_piece_id
        ORDER BY
            r.date DESC;
        "#,
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| format!("Failed to load data: {}", e))?;

    // Organize the data into nested structs
    let mut regimen_map = std::collections::HashMap::new();

    for row in results {
        let regimen = regimen_map
            .entry(row.regimen_id)
            .or_insert_with(|| PracticeRegiment {
                id: Some(row.regimen_id),
                date: row.regimen_date,
                pieces: vec![],
            });

        if let Some(piece_id) = row.piece_id {
            let piece = regimen.pieces.iter_mut().find(|p| p.id == Some(piece_id));

            if let Some(existing_piece) = piece {
                if let Some(log_id) = row.log_id {
                    existing_piece.logs.push(PracticeLog {
                        id: Some(log_id),
                        practice_piece_id: piece_id,
                        bpm: row.log_bpm.unwrap_or_default(),
                        timestamp: row.log_timestamp.unwrap(),
                    });
                }
            } else {
                let mut new_piece = PracticePiece {
                    id: Some(piece_id),
                    name: row.piece_name.unwrap_or_default(),
                    logs: vec![],
                };

                if let Some(log_id) = row.log_id {
                    new_piece.logs.push(PracticeLog {
                        id: Some(log_id),
                        practice_piece_id: piece_id,
                        bpm: row.log_bpm.unwrap_or_default(),
                        timestamp: row.log_timestamp.unwrap(),
                    });
                }

                regimen.pieces.push(new_piece);
            }
        }
    }

    let regimens: Vec<PracticeRegiment> = regimen_map.into_values().collect();

    Ok(serde_json::to_string(&regimens).map_err(|e| format!("Serialization failed: {}", e))?)
}

#[command]
async fn load_practice_regiments(pool: tauri::State<'_, SqlitePool>) -> Result<String, String> {
    // Fetch practice regiments and their associated piece names
    // Fetch regiments and their associated pieces
    let rows = sqlx::query!(
        r#"
            SELECT pr.id as "regiment_id!", pr.date as "date!", pp.id as "piece_id!", pp.name as "piece_name!"
            FROM practice_regiment pr
            INNER JOIN practice_piece pp ON pr.id = pp.practice_regiment_id
            ORDER BY pr.date DESC
        "#
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| format!("Failed to load data: {}", e))?;

    // Create a hashmap to group regiments and their pieces
    let mut regiments_map: HashMap<i64, PracticeRegiment> = HashMap::new();

    // Iterate through the results and group the pieces by regiment
    for row in rows {
        let regiment_id = row.regiment_id;
        let date = row.date;
        // Parse `regiment_date` from String to NaiveDateTime

        // Get or insert the regiment
        let regiment = regiments_map
            .entry(regiment_id)
            .or_insert(PracticeRegiment {
                id: Some(regiment_id),
                date, // Non-nullable NaiveDateTime
                pieces: Vec::new(),
            });

        // Add the piece to the regiment
        regiment.pieces.push(PracticePiece {
            id: Some(row.piece_id), // Non-nullable i64
            name: row.piece_name,   // Non-nullable String
            logs: Vec::new(),
        });
    }
    let regiments_vec: Vec<PracticeRegiment> = regiments_map.into_values().collect();

    let json_data = json!(regiments_vec).to_string();
    Ok(json_data) // Return the list of formatted strings
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env file
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let active_piece: Arc<Mutex<Option<i64>>> = Arc::new(Mutex::new(None)); // Store the active piece ID

    // Create the SQLite connection pool
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create SQLite pool");
    let _result = sqlx::query!("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("Failed to enable foreign keys");
    tauri::Builder::default()
        .manage(pool.clone()) // Manage the SQLite pool in Tauri state
        .manage(active_piece.clone())
        .setup(move |app| {
            let handle = app.handle().clone();
            let active_piece = active_piece.clone(); // Clone to pass into the async task

            task::spawn(async move {
                run_mqtt(handle, active_piece, pool.clone()).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            mark_active_piece,
            get_active_piece,
            create_full_regiment,
            load_practice_regiments,
            load_practice_regiments_2,
        ]) // Register the command
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn insert_practice_regiment_with_transaction(
    pool: &SqlitePool,
    date: NaiveDateTime,
    piece_names: Vec<String>,
) -> Result<(), Error> {
    let mut transaction = pool.begin().await?;
    let date_str = date.format("%Y-%m-%d %H:%M:%S").to_string();
    // Insert into practice_regiment table within transaction

    let result = sqlx::query!(
        "INSERT INTO practice_regiment (date) VALUES (?)",
        date_str // Ensure we're passing a string format date
    )
    .execute(&mut transaction) // Use the transaction here
    .await;

    match result {
        Ok(_) => println!("Successfully inserted practice_regiment"),
        Err(e) => {
            println!("Failed to insert practice_regiment: {:?}", e);
            return Err(e);
        }
    }
    let regiment_id: i32 = sqlx::query_scalar!("SELECT last_insert_rowid()")
        .fetch_one(&mut transaction)
        .await?;

    println!("Last inserted regiment_id: {}", regiment_id);
    // Insert each piece name within transaction
    for name in piece_names {
        sqlx::query!(
            "INSERT INTO practice_piece (practice_regiment_id, name) VALUES (?, ?)",
            regiment_id,
            name
        )
        .execute(&mut transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok(())
}

async fn run_mqtt(
    handle: tauri::AppHandle,
    active_piece: Arc<Mutex<Option<i64>>>,
    pool: SqlitePool,
) {
    // Set up MQTT client options
    let mut mqttoptions = MqttOptions::new("tauri-app", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    // Create the client synchronously
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Subscribe to the topic where you send BPM data (this is async)
    client
        .subscribe("esp32/midi", QoS::AtLeastOnce)
        .await
        .expect("Failed to subscribe");

    println!("Subscribed to MQTT topic");
    while let Ok(notification) = eventloop.poll().await {
        match notification {
            Event::Incoming(Packet::Publish(publish)) => {
                let msg = &publish.payload;
                if let Ok(msg_str) = String::from_utf8(msg.to_vec()) {
                    if let Ok(parsed_bpm) = msg_str.parse::<i64>() {
                        println!("Received and parsed BPM: {}", parsed_bpm);
                        handle.emit("mqtt_bpm", parsed_bpm).unwrap();
                        let active_piece_id = *active_piece.lock().unwrap(); // Safely lock the active piece

                        if let Some(active_id) = active_piece_id {
                            // Log the active practice piece
                            println!("Active practice piece ID: {}", active_id);

                            let latest_entry_result = sqlx::query!(
                                                r#"
                                                    SELECT timestamp as "timestamp: NaiveDateTime", bpm
                                                    FROM practice_piece_log_entry
                                                    WHERE practice_piece_id = ?
                                                    ORDER BY timestamp DESC
                                                    LIMIT 1
                                                "#,
                                                active_id
                                            )
                                            .fetch_optional(&pool)
                                            .await;

                            // Handle the Result from fetch_optional()
                            match latest_entry_result {
                                Ok(latest_entry) => {
                                    let now = Utc::now().naive_utc();
                                    let mut should_log = false;

                                    if let Some(record) = latest_entry {
                                        let last_timestamp = record.timestamp.unwrap();
                                        let last_bpm = record.bpm.unwrap();

                                        // Check if BPM has changed or if more than 5 minutes have passed
                                        if parsed_bpm != last_bpm
                                            || now.signed_duration_since(last_timestamp)
                                                > TimeDelta::minutes(5)
                                        {
                                            should_log = true;
                                        }
                                    } else {
                                        // No log entry exists, so we should log
                                        should_log = true;
                                    }

                                    if should_log {
                                        // Insert a new log entry
                                        sqlx::query!(
                                            "INSERT INTO practice_piece_log_entry (practice_piece_id, bpm, timestamp)
                                            VALUES (?, ?, ?)",
                                            active_id,
                                            parsed_bpm,
                                            now
                                        )
                                        .execute(&pool)
                                        .await
                                        .unwrap();

                                        println!(
                                            "Logged new practice piece entry for piece ID: {}",
                                            active_id
                                        );
                                    }
                                }
                                Err(e) => {
                                    println!("Failed to load latest entry: {}", e);
                                }
                            }
                        } else {
                            println!("No active practice piece.");
                        }
                    } else {
                        println!("Failed to parse BPM: {}", msg_str);
                    }
                } else {
                    println!("Received non-UTF8 MQTT message");
                }
            }
            _ => {}
        }
    }
}
