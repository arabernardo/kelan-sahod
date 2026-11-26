# Kelan Sahod

*Kelan* = when, *sahod* = salary. A small offline desktop app that counts the days until your next payday, in Taglish.

- **Frontend:** Vite + Vue 3 (`<script setup>`) + TypeScript, plain CSS
- **Desktop shell and logic:** Tauri 2 + Rust (`chrono`, `chrono-tz`, `tauri-plugin-store`)
- **Package manager:** npm
- No server, no network calls, no login. Everything runs on your machine.

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
  src/commands.rs       Thin Tauri command: gets today in Asia/Manila, calls payday.rs
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

## Update the holiday list every year

> **UPDATE `src-tauri/data/holidays.json` EVERY YEAR based on the official Presidential Proclamations.**

Holidays come from two places:

1. **Fixed-date holidays** are generated in Rust for any year (`fixed_holidays` in `holidays.rs`): Jan 1, Apr 9, May 1, Jun 12, Aug 21, Nov 1, Nov 30, Dec 8, Dec 25, Dec 30, Dec 31. Nothing to do here.
2. **Movable / proclaimed holidays** are a manual list in `holidays.json`: National Heroes Day, Maundy Thursday, Good Friday, Black Saturday, Eid'l Fitr, Eid'l Adha, Chinese New Year, and any special non-working day declared for the year.

To update:

1. Open the new year's Proclamation (Office of the President / Official Gazette).
2. Add one entry per holiday to `holidays`: `{ "date": "YYYY-MM-DD", "name": "..." }`.
3. Count **regular holidays** and **special non-working days** (banks are closed on both). Do **not** add **special working days**.
4. If you are not sure about a date, do **not** guess. Put it in `todoVerify` instead (it is never used in calculations).
5. Run `cargo test`. A test checks that the file parses and has no duplicate dates.
6. Rebuild the app. The file is embedded in the binary, so users never edit it.

If the current year has no entry in `holidays`, the home screen shows a small hint (*"Baka kulang pa ang holiday list para sa taon na 'to."*).

## Payday rule

The nominal payday is the date from the schedule (a day of 29-31 means the last day of a shorter month). If it falls on a **Saturday, Sunday or holiday**, the actual payday is the **nearest previous working day** (Mon-Fri and not a holiday). Paydays only ever move earlier, even into the previous month or year.

The next payday is found by adjusting every candidate first (previous month, this month, next two months) and picking the earliest **actual** date that is today or later.
