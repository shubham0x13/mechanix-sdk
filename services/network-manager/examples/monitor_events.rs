use futures::StreamExt;
use network_manager::{NetworkManager, NetworkManagerEvent};
use tokio::pin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nm = NetworkManager::new().await?;
    let events = nm.stream_events();

    pin!(events);

    println!("Listening for Network Manager events. Press Ctrl+C to exit.");

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\nStopping event monitor...");
                break;
            }
            Some(event) = events.next() => {
                print_event(&nm, event).await;
            }
        }
    }

    Ok(())
}

async fn print_event(nm: &NetworkManager, event: NetworkManagerEvent) {
    match event {
        NetworkManagerEvent::AccessPointAdded {
            device_path,
            access_point,
        } => {
            println!(
                "[Event] Access point added: {} on device {}",
                nm.get_access_point_ssid(&access_point)
                    .await
                    .unwrap_or_default(),
                device_path
            );
        }
        NetworkManagerEvent::AccessPointRemoved {
            device_path,
            access_point,
        } => {
            println!(
                "[Event] Access point removed: {} on device {}",
                nm.get_access_point_ssid(&access_point)
                    .await
                    .unwrap_or_default(),
                device_path
            );
        }
        _ => {
            println!("[Event] -> {:#?}", event);
        }
    }
}
