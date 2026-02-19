# 4ccountant

Desktop expense tracking and classification app built with Rust, Tauri v2, Svelte 5, and SQLite.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (v18+)
- Tauri v2 system dependencies — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

## Setup

```bash
git clone <repo-url>
cd 4ccountant
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

## Tests

```bash
cargo test
```
