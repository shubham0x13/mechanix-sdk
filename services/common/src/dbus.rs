use async_stream::stream;
use futures::{Stream, StreamExt};
use tokio::time::{Duration, sleep};
use zbus::{Connection, MatchRule, Message, MessageStream};

const RECONNECT_INITIAL_BACKOFF_MS: u64 = 250;
const RECONNECT_MAX_BACKOFF_MS: u64 = 5_000;

/// Creates a resilient, auto-reconnecting stream of domain events from D-Bus.
///
/// * `E` - The domain-specific Event type (e.g., BluetoothEvent)
/// * `F` - A parser function that converts a raw D-Bus Message into a Vec of Events
pub fn create_event_stream<E, F>(
    connection: Connection,
    rule: MatchRule<'static>,
    mut parser: F,
) -> impl Stream<Item = E> + Send + 'static
where
    E: Send + 'static,
    F: FnMut(&Message) -> Vec<E> + Send + 'static,
{
    stream! {
        let mut reconnect_backoff = Duration::from_millis(RECONNECT_INITIAL_BACKOFF_MS);

        loop {
            let mut dbus_stream = match MessageStream::for_match_rule(rule.clone(), &connection, None).await {
                Ok(stream) => {
                    reconnect_backoff = Duration::from_millis(RECONNECT_INITIAL_BACKOFF_MS);
                    stream
                }
                Err(err) => {
                    eprintln!("D-Bus stream creation failed: {err}. Retrying...");
                    sleep(reconnect_backoff).await;
                    reconnect_backoff = (reconnect_backoff * 2).min(Duration::from_millis(RECONNECT_MAX_BACKOFF_MS));
                    continue;
                }
            };

            while let Some(msg_result) = dbus_stream.next().await {
                match msg_result {
                    Ok(msg) => {
                        // Call the domain-specific parser passed in by the caller
                        for event in parser(&msg) {
                            yield event;
                        }
                    }
                    Err(err) => {
                        eprintln!("D-Bus stream error: {err}. Reconnecting.");
                        break;
                    }
                }
            }

            sleep(reconnect_backoff).await;
            reconnect_backoff = (reconnect_backoff * 2).min(Duration::from_millis(RECONNECT_MAX_BACKOFF_MS));
        }
    }
}
