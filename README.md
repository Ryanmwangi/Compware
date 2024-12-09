# CompareWare

CompareWare is an open-source platform for comparing tools (software, hardware, etc.) with structured, crowdsourced data. It combines **Leptos** for a modern, reactive frontend and **Nostr** for decentralized data storage.

## **Features**
- **Item Management**: Add, view, and manage items with metadata and key-value tags.
- **Nostr Integration**: 
  - Store and share data as Nostr events.
  - Authenticate users with Nostr keys.
- **Future Features**: Reviews and a Web of Trust for collaborative insights.


## **Getting Started**

### Prerequisites
- Rust (latest stable version)
- Leptos framework

### Installation
1. Clone the repository:
   ```bash
   git clone https://forge.ftt.gmbh/ryanmwangi/Compware.git
   cd compareware
   ```
2. Run the development server:
   ```bash
   cargo leptos serve
   ```
3. Open your browser at [http://localhost:3000](http://localhost:3000)

## **Roadmap**

1. **Item Management** (In progress)
   - Implement a form (`item_form.rs`) to allow users to add new items with metadata and key-value tags.
   - Create a listing component (`items_list.rs`) to display and manage added items.
   - Add backend functionality to validate and persist items using the Leptos framework.

2. **Review System** (To be implemented)
   - Design a data model to handle reviews, including ratings, comments, and reviewer metadata.
   - Build a user interface for submitting and viewing reviews linked to specific items.
   - Integrate filters and sorting to display reviews based on relevance and ratings.

3. **Nostr Integration** (To be integrated)
   - Use Nostr events for decentralized data storage, mapping item data to specific Nostr event types.
   - Authenticate users through their Nostr keys for secure and decentralized access.
   - Enable data sharing and synchronization with Nostr-compatible clients.