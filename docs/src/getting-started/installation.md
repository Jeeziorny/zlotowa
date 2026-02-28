# Installation

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (for building the backend)
- [Node.js](https://nodejs.org/) (for building the frontend)
- npm (comes with Node.js)

## Build from source

```bash
# Clone the repository
git clone https://github.com/kamiljeziorny/zlotowa.git
cd zlotowa

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Or build a release binary
npm run tauri build
```

The release binary will be in `src-tauri/target/release/`.

## Data storage

Your database is stored at:

| OS | Path |
|---|---|
| Linux | `~/.local/share/zlotowa/zlotowa.db` |
| macOS | `~/Library/Application Support/zlotowa/zlotowa.db` |
| Windows | `C:\Users\<you>\AppData\Roaming\zlotowa\zlotowa.db` |

The database is created automatically on first launch. To start fresh, delete the file.
