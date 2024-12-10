use nostr_sdk::client::Error;
use nostr_sdk::prelude::*;
use tokio::sync::mpsc;
use nostr_sdk::RelayPoolNotification;

pub struct NostrClient {
    client: Client,
}

impl NostrClient {
    pub async fn new(relay_url: &str) -> Result<Self, Error> {
        let keys = Keys::generate_from_os_random();
        let client = Client::new(&keys);
        client.add_relay(relay_url, None).await?;
        client.connect().await;

        Ok(Self { client })
    }

    pub async fn publish_item(
        &self,
        name: String,
        description: String,
        tags: Vec<(String, String)>
    ) -> Result<(), Error> {
        let content = serde_json::json!({
            "name": name,
            "description": description,
            "tags": tags
        });
        self.client.publish_text_note(content.to_string(), &[]).await?;
        Ok(())
    }

    pub async fn subscribe_to_items(
        &self,
        tx: mpsc::Sender<String>
    ) -> Result<(), Error> {
        let mut notifications = self.client.notifications();
        tokio::spawn(async move {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event(_url, event) = notification {
                    let content = event.content.clone();
                    tx.send(content).await.unwrap();
                }
            }
        });

        Ok(())
    }
}
