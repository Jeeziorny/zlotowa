# Task 98: Remove Unnecessary Clones

## Goal

Eliminate unnecessary `.clone()` calls in the IPC layer and classifier pipeline where owned values can be moved.

## Deliverables

### 1. IPC layer — `src-tauri/src/lib.rs`

**`add_expense()` (~line 108-111):** `input.title.clone()` and `input.category.clone()` are unnecessary — the `input` struct is owned and not used after constructing the Expense. Move the fields directly.

**`update_expense()` (~line 142-145):** Same pattern. Move `input.title` and `input.category` instead of cloning.

### 2. Classifier pipeline — `crates/core/src/classifiers.rs:91`

`expense.clone()` in the classification loop. Check whether the expense can be moved into the result tuple, or whether the loop needs the value again on the next iteration. If the clone is necessary (expense tested against multiple classifiers), document why with a comment.

### 3. Scan for other unnecessary clones

Grep for `.clone()` in non-test Rust code and flag any that clone owned values immediately before consuming them.

## Files to modify
- `src-tauri/src/lib.rs`
- `crates/core/src/classifiers.rs`

## Notes
- Don't remove clones that are genuinely needed (e.g., values used after the clone point, values passed to multiple consumers).
- Run `cargo clippy --workspace` after changes to verify no new warnings.
