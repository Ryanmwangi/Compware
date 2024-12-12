use nostr_sdk::client::Error;
use nostr_sdk::prelude::*;
use nostr_sdk::RelayPoolNotification;

pub struct NostrClient {
    client: Client,
}

impl NostrClient {
    pub async fn new(relay_url: &str) -> Result<Self, Error> {
        let keys = Keys::new(SecretKey::generate());
        let client = Client::new(keys);
        client.add_relay(relay_url).await?;
        client.connect().await;

        Ok(Self { client })
    }

    pub async fn publish_item(
        &self,
        name: String,
        description: String,
        tags: Vec<(String, String)>
    ) -> Result<(), Error> {
        let content = format!("{{\"name\":\"{}\",\"description\":\"{}\",\"tags\":[{}]}}", name, description, tags.iter().map(|(k, v)| format!("(\"{}\",\"{}\")", k, v)).collect::<Vec<_>>().join(","));
        let event = EventBuilder::new(Kind::TextNote, content).build()?;
        self.client.send_event_builder(event).await?;
        Ok(())
    }

    pub async fn subscribe_to_items(
        &self,
        tx: std::sync::mpsc::Sender<String>
    ) -> Result<(), Error> {
        let mut notifications = self.client.notifications();
        std::thread::spawn(move || {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { relay_url, subscription_id, event } = notification {
                    let content = event.content.clone();
                    tx.send(content).await.unwrap();
                }
            }
        });

        Ok(())
    }
}
