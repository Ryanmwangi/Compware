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

2. **Nostr Integration** (In progress)
   - Use Nostr events for decentralized data storage, mapping item data to specific Nostr event types.
   - Authenticate users through their Nostr keys for secure and decentralized access.
   - Enable data sharing and synchronization with Nostr-compatible clients.


## **Compareware: Next Steps**

Here’s how I intend to break down the vision into actionable steps to build upon the current codebase I’ve already built:

### **Immediate Steps**

#### **Basic Interface (Spreadsheet-like):**
- Create a grid-based UI to represent items and their properties.
- Use rows for properties and columns for items.
- Leverage a Leptos-based table or a custom grid component for rendering.

#### **Autocompletion for Adding Items and Properties:**
- Integrate Wikidata's search API to provide autocompletion for item and property inputs.
- Add a fallback to redirect users to the Wikidata item creation page when a search fails.

#### **Fetching Basic Information:**
- Use Wikidata's REST API to fetch metadata for newly added items (e.g., description, tags, etc.).
- Populate these fields in the spreadsheet automatically after adding an item.

---

### **Building on the Current Code**

#### **Enhance the `ItemForm` to Allow:**
- Searching for existing items via Wikidata.
- Displaying fetched details in the form.
- Modify the `handle_submit` function to fetch and populate additional item details after submission.

#### **Update the `App` Component to:**
- Add a placeholder grid view using Leptos’ `view!` macro.
- Render the comparison grid.
- Add functionality to fetch items' properties dynamically from Wikidata.

#### **Add Wikidata Autocompletion:**
- Use Gloo's HTTP client to make calls to the Wikidata search API.

---

### **Mid-Term Enhancements**

#### **Editable Fields with Wikidata Sync:**
- Implement field-level editing in the grid.
- Use Wikidata's APIs to update data directly for logged-in users.

#### **Subjective Properties with Nostr Integration:**
- Add a toggle for "objective" (Wikidata) vs. "subjective" (Nostr-backed) properties.
- Store subjective properties locally first and publish them to a Nostr relay for decentralized edits.

#### **Cache Mechanism:**
- Use a lightweight database (e.g., SQLite or a key-value store like Redis) as a cache for frequently accessed items and properties.
- Implement cache invalidation for edits to ensure the latest data is fetched.

---

### **Advanced Features**

#### **Advanced Filtering and Sorting:**
- Add functionality to filter items by tags or properties.
- Enable sorting by property values.

#### **Item Suggestions:**
- Based on properties and tags, suggest items for comparison.

#### **Collaborative Comparison:**
- Enable real-time collaboration with WebSockets, allowing users to view and edit comparisons together.

#### **Export/Share Comparison:**
- Add options to export the comparison as a CSV or share it via a unique link.
