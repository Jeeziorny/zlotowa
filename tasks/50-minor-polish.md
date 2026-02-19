# 50 — Minor Polish & Hardcoded Values

## Problem

Small issues that don't warrant individual tasks but should be cleaned up.

### Hardcoded values

1. **`src/lib/Sidebar.svelte:49`** — Version `v0.1.0` hardcoded inline. Should source from `package.json` or a shared constant.
2. **`src/lib/AddExpense.svelte:47`** and **`src/lib/ExpenseList.svelte:103`** — Debounce delay `300` duplicated. Extract to a shared constant.
3. **`src/lib/ExpenseList.svelte:19,621`** — Page size `50` and options `[25, 50, 100]` hardcoded.
4. **`src/lib/widgets/MonthlyTrend.svelte:14`** — `.slice(-6)` for trend period hardcoded.
5. **`src/lib/widgets/MostFrequent.svelte:11`** — `.slice(0, 5)` for top-N hardcoded.
6. **`src/lib/Settings.svelte:99`** — API key masking magic numbers (`length < 8`, `slice(0,4)`, `slice(-4)`).

### Date staleness in desktop app

7. **`src/lib/DatePicker.svelte:16-17`** — `new Date()` evaluated once at mount. If the app stays open across midnight, "today" is stale.
8. **`src/lib/widgets/BiggestExpense.svelte:4-7`** — Same issue: `let now = new Date()` computed once, stale across midnight.

### Over-broad visibility

9. **`src-tauri/src/lib.rs:14`** — `pub struct AppState` could be `pub(crate)`
10. **`src-tauri/src/lib.rs:20-66`** — IPC DTOs (`ExpenseInput`, `LlmConfigInput`, etc.) are `pub` but only used within the module. Could be `pub(crate)`.
11. **`crates/core/src/llm.rs:151,234,322`** — Provider structs (`OpenAiProvider`, `AnthropicProvider`, `OllamaProvider`) are `pub` but only instantiated inside `create_provider()`. Could be `pub(crate)` or private.

## Scope

- Extract shared constants for debounce, page sizes, trend window, top-N
- Source version from a central location
- Add named constants for API key masking thresholds
- Fix date staleness: recalculate `today` on component visibility or use a reactive timer
- Tighten `pub` visibility where only used within the crate
