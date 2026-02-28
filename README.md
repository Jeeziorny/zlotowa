# złotówa

Desktop expense tracking and classification app built with Rust, Tauri v2, Svelte 5, and SQLite.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (v18+)
- Tauri v2 system dependencies — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

## Setup

```bash
git clone <repo-url>
cd zlotowa
npm install
```

## Run

```bash
npm run tauri dev
```

This starts both the Vite dev server and the Rust backend. The app window opens automatically.

## Build

```bash
npm run tauri build
```

The release binary will be in `src-tauri/target/release/`.

## Updating

Your data (expenses, categories, budgets, rules) is stored in a SQLite database at:

```
~/Library/Application Support/zlotowa/zlotowa.db
```

This is separate from the application binary, so **updates never touch your data**.

To update to a newer version:

1. Pull the latest changes and rebuild:
   ```bash
   git pull
   npm install
   npm run tauri build
   ```
2. Replace the app in `/Applications/` with the new build from `src-tauri/target/release/bundle/macos/`.
3. Launch the app — schema migrations run automatically on startup, adding any new tables or columns without affecting existing data.

There is no auto-update mechanism. Updates are manual rebuild-and-reinstall.

## Tests

```bash
cargo test
```
