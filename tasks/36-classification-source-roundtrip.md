# 36 — Fix ClassificationSource String Roundtrip

## Problem

`ClassificationSource` in `models.rs:37-44`:
- `to_string()` (via Display/Debug) produces `"Llm"`, `"Database"`, `"Manual"`
- `from_str_opt()` accepts `"llm"`, `"database"`, `"manual"` (lowercase)
- Roundtrip fails: `ClassificationSource::Llm.to_string()` → `"Llm"` → `from_str_opt("Llm")` → `None`

Also in `src-tauri/src/lib.rs:335` where `ClassificationSource::Llm.to_string()` is stored to DB, but reads back through `from_str_opt()` which expects lowercase.

## Scope

- Make `from_str_opt()` case-insensitive (or normalize both sides to lowercase)
- Add a test verifying roundtrip for all variants
