use rumqttc::{AsyncClient, Client, Event, MqttOptions, Packet, QoS};
use std::time::Duration;
use tauri::{Emitter, Manager};
use tokio::task;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Run the MQTT client in the background using tokio's async runtime
            let handle = app.handle().clone();
            task::spawn(async move {
                run_mqtt(handle).await; // Make sure to call run_mqtt asynchronously
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
