# Task 79 — Remember CSV Column Mappings

Item: #17 (column mappings not persisted).

## Problem

Every CSV upload requires re-mapping columns (title, amount, date) even when uploading from the same bank every month. The column mapping step auto-detects by header name, but when headers don't match heuristics, the user must manually assign columns each time.

## Solution

### Persist mappings

When the user confirms column mapping and proceeds to step 3 (title cleanup), save the mapping to the `config` table.

**Config key:** `column_mappings`
**Config value:** JSON array of mapping objects:
```json
[
  {
    "pattern": "statement_",
    "headers": ["Date", "Description", "Amount", "Balance"],
    "mapping": { "title": 1, "amount": 2, "date": 0 },
    "dateFormat": "DD/MM/YYYY",
    "savedAt": "2025-12-15T10:30:00Z"
  }
]
```

### Filename pattern matching

Extract a pattern from the filename by stripping trailing digits and dates:
- `statement_2025_01.csv` → `statement_`
- `transactions-jan-2025.csv` → `transactions-`
- `export_20250115_123456.csv` → `export_`

Also store the exact header row for matching. Matching priority:
1. Exact header row match (strongest signal — same bank, same export format)
2. Filename pattern match (fallback)

### Restore flow

On step 2 (ColumnMapping) load:
1. Read `column_mappings` from config.
2. Check if the current file's headers exactly match any saved entry.
3. If yes, pre-apply the saved column assignments and date format.
4. Show a dismissible info banner: `"Column mapping restored from a previous upload."` with a `"Reset"` link that clears the restored mapping and falls back to auto-detection.
5. If no match, proceed with the existing auto-detection heuristics.

### Storage limits

Keep only the 10 most recent mapping entries (FIFO eviction by `savedAt`). This prevents unbounded config growth.

### Implementation

All changes are frontend-side — the `config` table already supports arbitrary key-value pairs via `get_config`/`save_config` IPC commands. No backend changes needed.

The mapping save happens in `BulkUpload.svelte` (or `ColumnMapping.svelte` via callback) when transitioning from step 2 → step 3. The mapping load happens in `ColumnMapping.svelte` on mount.

## Files

| File | Action |
|------|--------|
| `src/lib/bulk-upload/ColumnMapping.svelte` | Modify — load saved mappings on mount, show restore banner |
| `src/lib/BulkUpload.svelte` | Modify — save mapping on step 2→3 transition |

## Verification
1. Upload `statement_jan.csv`, manually map columns, proceed
2. Upload `statement_feb.csv` → columns auto-restored from saved mapping
3. "Reset" link clears restored mapping, falls back to auto-detect
4. Upload from a different bank → no match, normal auto-detect
5. After 11 different mappings saved, oldest is evicted
