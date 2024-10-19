use chrono::{Date, DateTime, Local, NaiveDateTime, ParseError};
use dotenv::dotenv;
use rumqttc::{AsyncClient, Client, Event, MqttOptions, Packet, QoS};
use sqlx::{Error, SqlitePool};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tauri::{command, Emitter, Manager};
use tokio::sync::Mutex;
use tokio::task;

const DATABASE_URL: &str = "sqlite:./practice.db";
#[derive(Debug)]
struct PracticePiece {
    name: String, // Each piece has a name, which could be NULL
}

#[derive(Debug)]
struct PracticeRegiment {
    id: i64,                    // Unique ID for the regiment
    pieces: Vec<PracticePiece>, // Vector of associated practice pieces
}

#[derive(Debug)]
struct PracticeRow {
    regiment_id: i64,
    piece_name: String, // This allows for NULL values in `name`
}

#[command]
async fn create_full_regiment(pool: tauri::State<'_, SqlitePool>) -> Result<(), String> {
    let date = Local::now().naive_local(); // Current time as NaiveDateTime
    let piece_names = vec![
        "Piece 1".to_string(),
        "Piece 2".to_string(),
        "Piece 3".to_string(),
    ];

    // Insert into the database
    match insert_practice_regiment_with_transaction(&pool, date, piece_names).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let error_message = format!("Failed to insert regiment in command: {:?}", e); // More detailed error
            println!("{}", error_message); // Log to console
            Err(error_message) // Return detailed error to the frontend
        }
    }
}

#[command]
async fn load_practice_regiments(
    pool: tauri::State<'_, SqlitePool>,
) -> Result<Vec<String>, String> {
    // Fetch practice regiments and their associated piece names
    // Fetch regiments and their associated pieces
    let rows = sqlx::query_as!(
        PracticeRow,
        r#"
            SELECT pr.id as "regiment_id!",  pn.name as piece_name
            FROM practice_regiment pr
            LEFT JOIN practice_piece pn ON pr.id = pn.practice_regiment_id
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

        // Parse `regiment_date` from String to NaiveDateTime

        // Get or insert the regiment
        let regiment = regiments_map
            .entry(regiment_id)
            .or_insert(PracticeRegiment {
                id: regiment_id,
                pieces: Vec::new(),
            });

        // Add the piece to the regiment
        regiment.pieces.push(PracticePiece {
            name: row.piece_name, // Non-nullable String
        });
    }

    // Convert the regiments into a vector of formatted strings
    let mut regiments = Vec::new();
    for (_id, regiment) in regiments_map {
        let mut regiment_info = format!("Regiment ID: {}", regiment.id);

        // Add piece names to the regiment's display
        for piece in regiment.pieces {
            regiment_info.push_str(&format!(
                "\n    Piece Name: {}",
                piece.name // Non-nullable String
            ));
        }

        regiments.push(regiment_info);
    }

    Ok(regiments) // Return the list of formatted strings
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env file
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

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
        .setup(|app| {
            let handle = app.handle().clone();

            task::spawn(async move {
                let pool = sqlx::sqlite::SqlitePool::connect("").await.unwrap();
                run_mqtt(handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_full_regiment,
            load_practice_regiments
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

async fn run_mqtt(handle: tauri::AppHandle) {
    // Set up MQTT client options
    let mut mqttoptions = MqttOptions::new("tauri-app", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    // Create the client synchronously
    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Subscribe to the topic where you send BPM data (this is async)
    client
        .subscribe("esp32/midi", QoS::AtLeastOnce)
        .await
        .expect("Failed to subscribe");

    println!("Subscribed to MQTT topic");
    while let Ok(notification) = eventloop.poll().await {
        println!("Received = {:?}", notification);
        match notification {
            Event::Incoming(Packet::Publish(publish)) => {
                let msg = &publish.payload;
                if let Ok(msg_str) = String::from_utf8(msg.to_vec()) {
                    if let Ok(parsed_bpm) = msg_str.parse::<f64>() {
                        println!("Received and parsed BPM: {}", parsed_bpm);
                        handle.emit("mqtt_bpm", parsed_bpm).unwrap();
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
