use nostr_sdk::{client::Error, prelude::*, RelayPoolNotification};
use tokio::sync::mpsc;
use std::fmt;

#[derive(Debug)]
pub enum MyError {
    NostrClientError(nostr_sdk::client::Error),
    NostrUnsignedError(nostr_sdk::event::unsigned::Error),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::NostrClientError(e) => write!(f, "Nostr Client Error: {}", e),
            MyError::NostrUnsignedError(e) => write!(f, "Nostr Unsigned Error: {}", e),
        }
    }
}

impl From<nostr_sdk::client::Error> for MyError {
    fn from(err: nostr_sdk::client::Error) -> MyError {
        MyError::NostrClientError(err)
    }
}

impl From<nostr_sdk::event::unsigned::Error> for MyError {
    fn from(err: nostr_sdk::event::unsigned::Error) -> MyError {
        MyError::NostrUnsignedError(err)
    }
}

pub struct NostrClient {
    client: Client,
    keys: Keys,
}

impl NostrClient {
    pub async fn new(relay_url: &str) -> Result<Self, Error> {
        let keys = Keys::new(SecretKey::generate());
        let client = Client::new(keys.clone());
        client.add_relay(relay_url).await?;
        client.connect().await;
        println!("Connected to relay: {}", relay_url);

        Ok(Self { client, keys })
    }

    pub async fn publish_item(
        &self,
        name: String,
        description: String,
        tags: Vec<(String, String)>
    ) -> Result<(), MyError> {
        let content = format!(
            "{{\"name\":\"{}\",\"description\":\"{}\",\"tags\":[{}]}}",
            name,
            description,
            tags.iter()
                .map(|(k, v)| format!("(\"{}\",\"{}\")", k, v))
                .collect::<Vec<_>>()
                .join(",")
        );

        // Create the event builder
        let event_builder = EventBuilder::new(Kind::TextNote, content.clone());
        // Build the unsigned event
        let unsigned_event = event_builder.build(self.keys.public_key());
        // Sign the event and handle the error explicitly
        let signed_event = match unsigned_event.sign(&self.keys).await {
            Ok(event) => event,
            Err(e) => return Err(MyError::from(e)), // Convert the error to the expected type
        };

        // Send the event
        self.client.send_event(signed_event.clone()).await?;
        println!("Event published: {:?}", signed_event);
        println!("Publishing content: {}", content);
        Ok(())
    }
    pub async fn subscribe_to_items(
        &self,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Error> {
        let mut notifications = self.client.notifications();
         // Use tokio::spawn for async context
         tokio::spawn(async move {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { relay_url: _, subscription_id: _, event } = notification {
                    let content = event.content.clone();
                    if tx.send(content).await.is_err() {
                        eprintln!("Failed to send message");
                    }
                }
            }
        });
        
        Ok(())
    }
}
