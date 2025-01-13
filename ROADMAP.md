
# CompareWare Roadmap

## **Current Features**

These features have been fully implemented:

### **Autocompletion for Adding Items and Properties**
- Integrated Wikidata's search API to provide autocompletion for item and property inputs.

### **Fetching Basic Information**
- Used Wikidata's REST API to fetch metadata for newly added items (e.g., description, tags).
- Automatically populated these fields in the spreadsheet after adding an item.

### **Wikidata Autocompletion**
- Used Gloo's HTTP client to make calls to the Wikidata search API.

### **App Component Updates**
- Added a placeholder grid view using Leptos’ `view!` macro.
- Rendered the comparison grid.
- Added functionality to fetch items' properties dynamically from Wikidata.

### **Enhance ItemForm**
- Enabled searching for existing items via Wikidata.
- Displayed fetched details in the form.

---

## **CompareWare: Next Steps**

### **Immediate Steps**

#### **Autocompletion for Adding Items and Properties:**
- Fetch all properties for items from wikidata.
- Autofill propertiy field with available properties for said item.
- Add a fallback to redirect users to the Wikidata item creation page when a search fails.

### **Authentication**
- Enable authentication for users using Nsec.app.

#### **Subjective Properties with Nostr Integration:**
- Add a toggle for "objective" (Wikidata) vs. "subjective" (Nostr-backed) properties.
- Store subjective properties locally first and publish them to a Nostr relay for decentralized edits.

#### **Cache Mechanism:**
- Use a lightweight database (e.g., SQLite or a key-value store like Redis) as a cache for frequently accessed items and properties.
- Implement cache invalidation for edits to ensure the latest data is fetched.

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
