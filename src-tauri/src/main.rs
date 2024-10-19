use chrono::{Date, DateTime, Local, NaiveDateTime};
use dotenv::dotenv;
use rumqttc::{AsyncClient, Client, Event, MqttOptions, Packet, QoS};
use sqlx::{Error, SqlitePool};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tauri::{command, Emitter, Manager};
use tokio::sync::Mutex;
use tokio::task;

const DATABASE_URL: &str = "sqlite:./practice.db";
#[derive(Debug, sqlx::FromRow)]
struct PracticeRegiment {
    id: i32,
    date: NaiveDateTime,
}
#[derive(Debug, sqlx::FromRow)]
struct PracticePiece {
    id: i32,
    practice_regiment_id: i32,
    name: String,
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
            let error_message = format!("Failed to insert regiment: {:?}", e); // More detailed error
            println!("{}", error_message); // Log to console
            Err(error_message) // Return detailed error to the frontend
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env file
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create the SQLite connection pool
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create SQLite pool");
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
        .invoke_handler(tauri::generate_handler![create_full_regiment]) // Register the command
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
    .execute(pool)
    .await;

    match result {
        Ok(_) => println!("Successfully inserted practice_regiment"),
        Err(e) => {
            println!("Failed to insert practice_regiment: {:?}", e);
            return Err(e);
        }
    }
    let regiment_id: i32 = sqlx::query_scalar!("SELECT last_insert_rowid()")
        .fetch_one(pool)
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
