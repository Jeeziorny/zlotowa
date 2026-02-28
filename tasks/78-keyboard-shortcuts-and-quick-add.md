# Task 78 — Keyboard Shortcuts & Quick-Add

Merges items: #15 (no keyboard shortcuts), #16 (adding expense requires 3 clicks).

## Problem

No global keyboard shortcuts exist — every action requires mouse navigation. Adding a single expense requires 3 clicks: sidebar → Expenses → + Add. For a desktop app, this is a significant workflow gap.

## Solution

### Global keyboard shortcuts

Add a `keydown` listener in `App.svelte` that handles:

| Shortcut | Action | Guard |
|----------|--------|-------|
| `Cmd+N` | Navigate to Add Expense | Not in input/textarea |
| `Cmd+U` | Navigate to Bulk Upload | Not in input/textarea |
| `Cmd+K` | Focus search on Expense List | Only when on expenses page |
| `Escape` | Close modal / go back from subview | Always |
| `Cmd+1` | Go to Dashboard | Not in input/textarea |
| `Cmd+2` | Go to Expenses | Not in input/textarea |
| `Cmd+3` | Go to Categories | Not in input/textarea |
| `Cmd+4` | Go to Budget | Not in input/textarea |

### Guard conditions

- Don't fire if `activeElement` is `input`, `textarea`, or `select` (except Escape, which should close modals regardless).
- Don't fire if a modal is open (except Escape to close it).
- Respect the navigation guard: if bulk upload is dirty, show the confirmation modal instead of navigating.
- Use `e.preventDefault()` to suppress browser defaults (Cmd+N opens new window, etc.).

### Sidebar quick-add button

Add a prominent `"+ Add"` button at the top of the sidebar navigation, above the page links:

```html
<button class="w-full bg-amber-500 hover:bg-amber-400 text-gray-950 rounded-lg py-2 text-sm font-medium mb-3 transition-colors"> <!-- per task #82 palette -->
  + Add
</button>
```

Clicking navigates directly to the add-expense form (`currentPage = "expenses"`, `subView = "add"`). This reduces the path from 3 clicks to 1 from any page.

### Shortcut hint overlay

Add a `?` button in the sidebar footer (next to the version label) that opens a keyboard shortcuts cheat sheet.

The overlay is a simple modal listing all shortcuts in a two-column layout:
```
Cmd+N    Add expense
Cmd+U    Bulk upload
Cmd+K    Search expenses
Cmd+1-4  Switch pages
Esc      Back / Close
```

Styled as a `bg-gray-900 border border-gray-800 rounded-xl p-6` modal, same as other dialogs. Closes on Escape or click-outside.

### Implementation notes

- Use `navigator.platform` or `navigator.userAgent` to detect macOS vs other (show `⌘` vs `Ctrl` in hints).
- The listener attaches in `onMount` and detaches in `onDestroy`.
- Shortcut handling should be a single function with a switch on `e.key` + modifier check.

## Files

| File | Action |
|------|--------|
| `src/App.svelte` | Modify — global keydown listener, shortcut routing |
| `src/lib/Sidebar.svelte` | Modify — add quick-add button, `?` shortcut hint button |
| `src/lib/KeyboardShortcuts.svelte` | Create — cheat sheet overlay |

## Verification
1. `Cmd+N` → navigates to Add Expense from any page
2. `Cmd+U` → navigates to Bulk Upload from any page
3. `Cmd+K` → focuses search input on expense list
4. `Cmd+1-4` → switches pages
5. `Escape` → closes modals, goes back from subviews
6. Shortcuts don't fire when typing in inputs
7. Sidebar `+ Add` button navigates to add form in one click
8. `?` button opens shortcut cheat sheet
