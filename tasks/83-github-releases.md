# Task 83 — GitHub Releases Distribution

Set up automated cross-platform builds and distribution via GitHub Releases so friends can download installers from a link.

## Problem

No distribution mechanism exists. Sharing the app requires building locally and sending binaries manually.

## Solution

### Prerequisites (manual, outside Claude Code)

1. **Create a GitHub repository** — push the codebase (public or private with collaborator access)
2. **Code signing (optional but recommended for macOS):**
   - Without signing: macOS users get "unidentified developer" warning, must right-click > Open
   - With signing: requires Apple Developer account ($99/yr), set `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` as GitHub secrets
3. **Windows signing (optional):** EV certificate or skip (users get SmartScreen warning on first run)

### Step 1 — GitHub Actions workflow

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: macos-latest
            target: aarch64-apple-darwin
            label: macOS-arm64
          - platform: macos-latest
            target: x86_64-apple-darwin
            label: macOS-x64
          - platform: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
            label: Linux-x64
          - platform: windows-latest
            target: x86_64-pc-windows-msvc
            label: Windows-x64

    runs-on: ${{ matrix.platform }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm

      - name: Install frontend deps
        run: npm ci

      - name: Install Linux dependencies
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf \
            libssl-dev \
            libgtk-3-dev

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          # macOS signing (uncomment if configured):
          # APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          # APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          # APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          # APPLE_ID: ${{ secrets.APPLE_ID }}
          # APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          # APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "zlotowa ${{ github.ref_name }}"
          releaseBody: "Download the installer for your platform below."
          releaseDraft: true
          prerelease: false
          args: --target ${{ matrix.target }}
```

Key points:
- Triggers on version tags (`v0.1.0`, `v0.2.0`, etc.)
- Builds for macOS (ARM + Intel), Linux, Windows
- `tauri-action` handles bundling and uploads artifacts to a draft GitHub Release
- Draft release lets you review before publishing
- `GITHUB_TOKEN` is auto-provided, no setup needed

### Step 2 — Add .gitignore entries

Ensure these are gitignored (some may already be):

```
# Build artifacts
/target/
/dist/
/node_modules/

# Personal files in repo root
*.pdf
*.epub
*.ics
*.csv
*.zip
*.html
*.svg
kamil-insights_files/
```

### Step 3 — Version bumping

Before tagging a release, version should be consistent across:

| File | Field |
|------|-------|
| `src-tauri/tauri.conf.json` | `"version"` |
| `src-tauri/Cargo.toml` | `version` |
| `package.json` | `"version"` |

All currently at `0.1.0`. For future releases, bump all three before tagging.

### Step 4 — Release process

```bash
# 1. Ensure clean state
git status  # should be clean

# 2. Tag the release
git tag v0.1.0
git push origin main --tags

# 3. Wait for GitHub Actions to build (~10-15 min)
# 4. Go to GitHub > Releases > edit the draft
# 5. Review artifacts, edit release notes if needed
# 6. Publish the release
# 7. Share the release URL with friends
```

### Output artifacts per platform

| Platform | Artifact |
|----------|----------|
| macOS | `.dmg` (drag to Applications) |
| Windows | `.msi` installer |
| Linux | `.AppImage` (portable) + `.deb` |

## Files

| File | Action |
|------|--------|
| `.github/workflows/release.yml` | Create — CI workflow |
| `.gitignore` | Modify — ensure build artifacts and personal files excluded |

## Verification

1. Push to GitHub with a `v0.1.0` tag
2. GitHub Actions triggers, all 4 matrix jobs pass (green)
3. Draft release appears with `.dmg`, `.msi`, `.AppImage`, `.deb` artifacts
4. Download macOS `.dmg` on another machine, open and run — app works
5. (If unsigned) macOS Gatekeeper shows "unidentified developer" — right-click > Open works

## Future enhancements (not in scope)

- **Auto-updater:** Tauri's `tauri-plugin-updater` checks a JSON endpoint for new versions
- **macOS universal binary:** combine ARM + Intel into one `.dmg`
- **Notarization:** `xcrun notarytool` step to avoid Gatekeeper entirely
- **Homebrew tap:** formula pointing to release artifacts
