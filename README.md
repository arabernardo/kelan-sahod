# Kelan Sahod

**Version 0.1.1**

*Kelan* = when, *sahod* = salary. A small offline desktop app that counts the days until your next payday, in Taglish.

- **Frontend:** Vite + Vue 3 (`<script setup>`) + TypeScript, plain CSS
- **Desktop shell and logic:** Tauri 2 + Rust (`chrono`, `chrono-tz`, `tauri-plugin-store`)
- **Package manager:** npm
- No server and no login. The app works fully offline. Its only network call is an optional download of the holiday list from GitHub (see below); no user data is ever sent.

## How it is built

```
src/                    Vue UI only (no date math)
  copy.ts               ALL user-facing text lives here
  types.ts              TypeScript mirror of the Rust response types
  api.ts                invoke() + settings store wrappers
  components/           SetupScreen, HomeScreen, Confetti
src-tauri/
  src/payday.rs         Pure payday logic + unit tests (today is a PARAMETER)
  src/holidays.rs       Fixed-date holidays + loader for the movable list
  src/commands.rs       Thin Tauri commands: gets today in Asia/Manila, calls payday.rs
  src/remote.rs         Optional holiday download from GitHub + on-disk cache (the only networking)
  data/holidays.json    Movable / proclaimed-yearly holidays (embedded in the binary)
  capabilities/         Tauri permissions (core + store)
```

The chosen schedule is saved by `tauri-plugin-store` in `settings.json` (in the app's data folder).

## Prerequisites

- **Node.js** 20 or newer, and npm
- **Rust** stable (install with [rustup](https://rustup.rs))
- **Tauri system dependencies** for your OS: <https://v2.tauri.app/start/prerequisites/>
  - **Windows:** Microsoft C++ Build Tools ("Desktop development with C++") and WebView2 (already on Windows 10/11)
  - **macOS:** Xcode Command Line Tools (`xcode-select --install`)
  - **Linux (Debian/Ubuntu):** `sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libxdo-dev librsvg2-dev pkg-config`

## Run

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

The installer ends up in `src-tauri/target/release/bundle/`. The icon in `src-tauri/icons/` is a placeholder. To make your own, replace `src-tauri/icons/app-icon.png` (square, 1024x1024) and run `npm run tauri icon src-tauri/icons/app-icon.png`. That also generates the `.icns` file needed for macOS bundles.

## Test

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # Rust: payday and holiday logic (fixed dates, no real clock)
npm test                                          # Vitest: copy helper
npm run typecheck                                 # vue-tsc
```

## Change the copy

Everything the user reads is in `src/copy.ts`: the countdown messages (2-3 variants per bucket, picked at random), the adjustment note, preset labels, and screen text. `{n}` in a message is replaced with the days left.

## Holidays: how the list stays current

Holidays come from two places:

1. **Fixed-date holidays** are generated in Rust for any year (`fixed_holidays` in `holidays.rs`): Jan 1, Apr 9, May 1, Jun 12, Aug 21, Nov 1, Nov 30, Dec 8, Dec 25, Dec 30, Dec 31. Nothing to do here.
2. **Movable / proclaimed holidays** live in `src-tauri/data/holidays.json`: National Heroes Day, Maundy Thursday, Good Friday, Black Saturday, Eid'l Fitr, Eid'l Adha, Chinese New Year, and any special non-working day declared for the year.

> **UPDATE `holidays.json` EVERY YEAR based on the official Presidential Proclamations.**

Count **regular holidays** and **special non-working days** (banks are closed on both). Do **not** add **special working days**. If you are not sure about a date, do **not** guess: put it in `todoVerify` (never used in calculations).

### Proclaimed late? The app syncs the list from GitHub

Some holidays are proclaimed only a week or so ahead, which is faster than any app release. So the app downloads the same `holidays.json` from GitHub:

- It checks at launch and then at most every 6 hours while open. The download happens in Rust (never in the webview), has an 8-second timeout and a 256 KB cap, and must be `https`.
- A downloaded file is used only if it passes strict validation and its `"updated"` date is the same or newer than the built-in list. A stale mirror can never overwrite a newer list, and a newer app release is never beaten by an old cache.
- Order of use: valid cached download, then the list built into the app. Offline, broken JSON, or a 404 simply means the app keeps using what it has, silently.
- The home screen shows "Holiday list as of {date}".

**One-time setup:** push this project to GitHub, then set `HOLIDAYS_URL` in `src-tauri/src/remote.rs` to the raw URL of `src-tauri/data/holidays.json` (replace `OWNER/REPO`, and the branch if it is not `main`). Until you do, sync is skipped and the app is fully offline.

**When a holiday is proclaimed:**

1. Edit `src-tauri/data/holidays.json` on GitHub: add `{ "date": "YYYY-MM-DD", "name": "..." }` to `holidays`.
2. Set `"updated"` to today's date. **This step is required**: files without a newer-or-equal `updated` date are ignored.
3. Commit. Installed apps pick it up on their next sync (GitHub's raw URLs cache for a few minutes).
4. Before releasing a new app version, run `cargo test` (it checks the file parses and passes the same validation the app applies to downloads).

### The "baka may bagong holiday" hint

- If the current year has no entries at all in the list, the home screen shows "Baka kulang pa ang holiday list para sa taon na 'to."
- If the next payday is 10 days away or closer, it shows "Baka may bagong holiday na hindi pa nakalista." unless the list was confirmed against GitHub within the last 24 hours. Offline users see it; synced users don't.

## Payday rule

The nominal payday is the date from the schedule (a day of 29-31 means the last day of a shorter month). If it falls on a **Saturday, Sunday or holiday**, the actual payday is the **nearest previous working day** (Mon-Fri and not a holiday). Paydays only ever move earlier, even into the previous month or year.

The next payday is found by adjusting every candidate first (previous month, this month, next two months) and picking the earliest **actual** date that is today or later.

## Changelog

### 0.1.1
- **Holiday sync from GitHub.** The app downloads the latest `holidays.json` in the background (at launch, then at most every 6 hours), validates it, and caches it on disk. Offline, the cached or built-in list is used. Setup: set `HOLIDAYS_URL` in `src-tauri/src/remote.rs`.
- **`updated` date in `holidays.json`.** Required. A downloaded list is only accepted if its `updated` date is the same or newer than the built-in one. Bump it on every edit.
- **Widened hint.** "Baka may bagong holiday na hindi pa nakalista." shows when payday is 10 days away or closer, unless the list was confirmed against GitHub in the last 24 hours.
- **"Holiday list as of {date}"** line on the home screen.
- New dependency: `ureq` (Rust). No new npm packages.
- Dev tooling (since 0.1.0): `vitest` ^3 and `@types/node` ^22.
- Note: this version adds the app's only network call, an optional download of the holiday file. No user data is sent.

### 0.1.0
- First version: setup screen, countdown, payday adjustment (previous working day), holiday handling, Taglish copy, light/dark mode.
