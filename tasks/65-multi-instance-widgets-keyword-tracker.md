# Task 65 — Multi-Instance Widgets & Keyword Expense Tracker

## Goal

Evolve the widget system from single-instance-per-type to support multiple instances of the same widget, each with its own configuration. Deliver a **Keyword Tracker** widget as the first configurable, multi-instance widget — lets the user pick a keyword (e.g. "LIDL") and see a monthly spending graph filtered to matching expenses.

## Motivation

Users want ad-hoc spending lenses without multi-category tagging. A keyword filter + graph is a lightweight alternative: no schema changes to expenses, no new classification logic. Multi-instance support is required so users can track several keywords simultaneously.

---

## Current State

- **Registry** (`src/lib/widgets/registry.js`): static array of `{ id, name, description, size, component }`.
- **Active widgets** stored as `Vec<String>` of widget IDs (JSON in `config` table, key `"active_widgets"`).
- **Dashboard** maps IDs → registry entries, renders `<widget.component {expenses} {onnavigate} />`.
- **IPC**: `get_active_widgets` returns `Option<Vec<String>>`, `save_active_widgets` accepts `Vec<String>`.
- **No per-widget config**, no instance concept, no duplicate IDs allowed.

---

## Design

### 1. Widget Instance Model

Replace the flat ID array with an array of instance objects:

```jsonc
// before
["total-stats", "monthly-trend"]

// after
[
  { "widgetId": "total-stats", "instanceId": "total-stats" },
  { "widgetId": "monthly-trend", "instanceId": "monthly-trend" },
  { "widgetId": "keyword-tracker", "instanceId": "kw-1", "config": { "keyword": "LIDL" } },
  { "widgetId": "keyword-tracker", "instanceId": "kw-2", "config": { "keyword": "AMAZON" } }
]
```

- `widgetId` — references the registry entry (component, size, name).
- `instanceId` — unique across the active list. For non-configurable widgets, equals `widgetId`. For multi-instance widgets, generated (e.g. `kw-<timestamp>`).
- `config` — optional object, only present for configurable widgets.

### 2. Registry Changes (`registry.js`)

Add optional flags to widget definitions:

```js
{
  id: "keyword-tracker",
  name: "Keyword Tracker",
  description: "Monthly spending for expenses matching a keyword.",
  size: "half",
  component: KeywordTracker,
  configurable: true,   // has per-instance config
  multiInstance: true,   // can appear multiple times
}
```

Existing widgets get neither flag — they behave exactly as today.

### 3. Migration of Stored Format

`get_active_widgets` currently returns `Option<Vec<String>>`. After this change it returns the new instance array format. Handle migration:

- On load, if the stored JSON is a string array (old format), convert each ID to `{ widgetId: id, instanceId: id }`.
- On save, always write the new format.

This is a **frontend-only migration** — the Rust IPC commands still store/retrieve an opaque JSON string via `get_config`/`set_config`. No Rust changes needed for the format itself.

### 4. Dashboard Changes (`Dashboard.svelte`)

| Area | Change |
|------|--------|
| **State** | `activeWidgetIds: string[]` → `activeInstances: WidgetInstance[]` |
| **Derived** | `activeWidgets` maps instances to registry entries, attaching instance config |
| **Add widget** | Non-multi-instance: same as today (add once, hide from picker). Multi-instance: opens a config dialog first (keyword input), then adds with generated instanceId |
| **Remove** | Removes by `instanceId` instead of `widgetId` |
| **Reorder** | Same swap logic, on `activeInstances` array |
| **Render** | Pass `config` prop to component: `<widget.component {expenses} {onnavigate} config={instance.config} />` |
| **Picker** | Non-multi-instance widgets that are already active: hidden (same as today). Multi-instance widgets: always shown in picker with "(+ add another)" hint |
| **Edit mode** | Multi-instance widgets show an edit (pencil) button to change config (keyword) |

### 5. Keyword Tracker Widget (`src/lib/widgets/KeywordTracker.svelte`)

**Props:** `{ expenses, config }` where `config = { keyword: string }`.

**Behavior:**
- Filter `expenses` where `title` contains `config.keyword` (case-insensitive).
- Group by month (`date.slice(0, 7)`), sum absolute amounts.
- Render bar chart (same CSS-bar technique as MonthlyTrend).
- Show keyword in widget header: "Keyword: LIDL".
- Show total matched amount and count below the chart.
- Empty state if no matches: "No expenses matching 'LIDL'."

**Config UI (inline or modal):**
- Text input for keyword.
- Shown when adding the widget and when clicking edit (pencil) in edit mode.
- Save updates the instance config in `activeInstances` and persists.

---

## Files to Change

| File | Change |
|------|--------|
| `src/lib/widgets/registry.js` | Add `configurable`/`multiInstance` flags to type. Add KeywordTracker entry. |
| `src/lib/widgets/KeywordTracker.svelte` | **New file.** Keyword filter + monthly bar chart. |
| `src/lib/Dashboard.svelte` | Instance model, format migration, config passing, picker logic for multi-instance, edit button for config. |

## Files That Do NOT Change

- **Rust backend** — IPC commands already store/retrieve opaque JSON. No changes to `lib.rs` or `db.rs`.
- **Existing widget components** — they receive an unused `config` prop at worst; no modifications needed.
- **Database schema** — config table already handles this.

---

## Backward Compatibility

- Old format (`["total-stats", ...]`) auto-migrates on first load.
- Existing widgets without `configurable`/`multiInstance` work exactly as before.
- If a user downgrades, the new JSON format would fail to parse in the old code — acceptable for a desktop app with no multi-version concerns.

---

## Testing

- **Manual:** Add/remove/reorder keyword tracker instances with different keywords. Verify persistence across app restarts. Verify old-format migration. Verify existing widgets unaffected.
- **Tauri IPC tests:** Existing `widget_config_save_and_load` test still passes (opaque JSON).

---

## Out of Scope

- Configurable options for existing widgets (date range, display mode) — future task.
- Drag-and-drop reordering — keep arrow buttons.
- Widget resizing — keep `size: "half"` / `"full"` from registry.
