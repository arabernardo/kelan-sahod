# Kelan Sahod

**Version 0.1.2**

**Author:** Arianna Maouna Bernardo

*Kelan* = when, *sahod* = salary. A small offline desktop app that counts the days until your next payday, in Taglish.

- **Frontend:** Vite + Vue 3 (`<script setup>`) + TypeScript, plain CSS
- **Desktop shell and logic:** Tauri 2 + Rust (`chrono`, `chrono-tz`, `tauri-plugin-store`)
- **Package manager:** npm
- No server and no login. The app works fully offline. Its only network call is an optional download of the holiday list from GitHub (see below); no user data is ever sent.

## How it is built

```
src/                    Vue UI only (no date math)
  copy.ts               ALL user-facing text lives here, in every language (Tagalog + English)
  types.ts              TypeScript mirror of the Rust response types
  api.ts                invoke() + settings store wrappers
  components/           SetupScreen, HomeScreen, SettingsScreen, Confetti
src-tauri/
  src/payday.rs         Pure payday logic + unit tests (today is a PARAMETER)
  src/holidays.rs       Generated + proclaimed holidays, with the loader for holidays.json
  src/layout.rs         Pure: Compact / Full layout modes and their window sizes
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

**"Port 1420 is already in use"?** The dev server needs a fixed port because Tauri reads it from `tauri.conf.json`. Either stop the old dev server (usually a leftover `tauri dev`; close it with Ctrl+C next time), or just run:

```bash
npm run tauri:dev
```

It picks the first free port starting at 1420 and gives the same port to both Vite and Tauri (`scripts/dev.mjs`, nothing in your config files changes). Extra flags are passed on, e.g. `npm run tauri:dev -- --release`. Add `--print` to see which port it would use without starting anything.

## Test a specific date (e.g. payday)

In dev mode you can make the app pretend it is another day, to see the payday screen (0 days left, the confetti burst, the "Today" message) without waiting.

**Easiest way, works in any terminal:**

```bash
npm run tauri:dev -- --date=2026-11-27
```

(The `--` is required: it passes the option on to the script.) Use the **payday date shown on the home screen** (the "Sa Biyernes, Nov 27" line) to land exactly on payday, or the day before it to see "1 day". The terminal prints `Test date: the app will pretend today is ...`, and then `[dev] KELAN_SAHOD_TODAY is set: pretending today is ...` when the app starts. If you don't see that second line, the date did not reach the app. Without `--date` the app uses the real date.

**Alternative: set the environment variable yourself** (the syntax depends on your terminal, so check which one you are in):

```powershell
# Windows PowerShell
$env:KELAN_SAHOD_TODAY = "2026-11-27"; npm run tauri dev
```
```bat
:: Windows CMD (use exactly this; a space before && is fine)
set KELAN_SAHOD_TODAY=2026-11-27 && npm run tauri dev
```
```bash
# macOS / Linux / Git Bash
KELAN_SAHOD_TODAY=2026-11-27 npm run tauri dev
```

A PowerShell command pasted into CMD (or the other way around) fails, and so does `set ...` in Git Bash. In VS Code, check the shell name at the top right of the terminal panel.

- A date that isn't a real `YYYY-MM-DD` date is rejected by `--date` and ignored by the variable (with a message in the terminal).
- It only exists in **debug builds**. It is compiled out of `npm run tauri build`, so an installed app always uses the real date in Asia/Manila.
- A variable you set yourself stays on for the whole terminal session. Clear it with PowerShell `Remove-Item Env:KELAN_SAHOD_TODAY`, CMD `set KELAN_SAHOD_TODAY=`, or open a new terminal. `--date` only affects that one run.

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

## Settings

Open **Settings** from the home screen.

- **Layout: Full** (default): a wide landscape window (960 x 640) with two columns: the countdown on the left, and progress, this month's holidays and the hints on the right.
- **Layout: Compact**: a small portrait window (380 x 540) with just the countdown, the payday date, the adjustment note, the message and the progress bar. It can't be dragged wider than 560 px, so it stays compact. Warnings about the holiday list still show.

The choice applies immediately, is saved in `settings.json` with your schedule, and is restored on the next launch (the window is sized before it is shown, so it never flashes at the wrong size). Window sizes live in `src-tauri/src/layout.rs`.

**Language: Tagalog or English.** Tagalog (default) is Filipino with a conyo / Taglish tone; English is plain and friendly. The whole app switches immediately, including the countdown messages, the "naka-advance" note, weekday names and the screen reader text. The choice is saved with your other settings. Month names are written the same way in both languages ("Dec 14"), and holiday names are shown as listed in the data.

**About** (in Settings) shows the version and the author. To change the author name: `src/copy.ts` (shown in the app), `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` (publisher and copyright).

## Change the copy

Everything the user reads is in `src/copy.ts`, in two blocks: `tl` (Tagalog) and `en` (English). Both have the same shape: the English block is typed against the Tagalog one, so a missing translation is a type error, and a test checks that no string is empty.

- **Countdown messages:** 2-3 variants per bucket (0, 1, 2-3, 4-7, 8-15, 16+ days), picked at random. `{n}` is replaced with the days left.
- **Adjustment note, weekday names, preset hints, screen text:** all in the same blocks.
- **Add a language:** add its code to `Language` in `src/types.ts`, write a full block like `tl` in `copy.ts`, add it to `copies`, and add a choice for it in `src/components/SettingsScreen.vue`.

## Holidays: how the list stays current

Holidays come from two places:

1. **Generated holidays: automatic, no proclamation needed.** Created in Rust for any year (`holidays.rs`):
   - Fixed dates: Jan 1, Apr 9, May 1, Jun 12, Aug 21, Nov 1, Nov 2, Nov 30, Dec 8, Dec 24, Dec 25, Dec 30, Dec 31.
   - Rule-based: **National Heroes Day** (the *last Monday* of August, so it is not a fixed date: Aug 31 in 2026, Aug 30 in 2027), and **Maundy Thursday, Good Friday and Black Saturday** (counted back from Easter Sunday).
2. **Movable / proclaimed holidays** live in `src-tauri/data/holidays.json`: **Eid'l Fitr, Eid'l Adha, Chinese New Year, Maundy Thursday, Good Friday**, and any special non-working day declared for the year.
   - **A JSON entry replaces the generated holiday with the same name in the same year** (names compared ignoring case). If a proclamation ever moves Good Friday, the JSON date wins and the computed one disappears for that year only.
   - The generated Holy Week dates are the fallback for years the JSON doesn't list yet.

> **Update `holidays.json` every year based on the official Presidential Proclamations.** Don't list fixed-date holidays or National Heroes Day there; the app generates them.

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

- If the current year has no entries at all in the proclaimed list (`holidays.json`), the home screen shows "Baka kulang pa ang holiday list para sa taon na 'to."
- If the next payday is 10 days away or closer, it shows "Baka may bagong holiday na hindi pa nakalista." unless the list was confirmed against GitHub within the last 24 hours. Offline users see it; synced users don't.

## Payday rule

The nominal payday is the date from the schedule (a day of 29-31 means the last day of a shorter month). If it falls on a **Saturday, Sunday or holiday**, the actual payday is the **nearest previous working day** (Mon-Fri and not a holiday). Paydays only ever move earlier, even into the previous month or year.

The next payday is found by adjusting every candidate first (previous month, this month, next two months) and picking the earliest **actual** date that is today or later.

## Changelog

### Unreleased
- **Payday screen:** on payday the home screen no longer shows the number 0; it shows "Sahod na!" / "Payday!" instead (confetti and the message stay), and the date line under it is hidden.
- **Test date for development:** `npm run tauri:dev -- --date=YYYY-MM-DD` (or the `KELAN_SAHOD_TODAY` variable) makes the dev app pretend it is that day, for example to see the payday screen. Debug builds only; see "Test a specific date".
- **Language switch: Tagalog / English** in Settings. All strings live in `src/copy.ts` in both languages; the choice applies immediately and is remembered.
- **`npm run tauri:dev`** finds a free port automatically when 1420 is busy (`scripts/dev.mjs`).
- **Landscape window.** The default window is now 960 x 640 with a two-column home screen and a two-column setup screen. Narrow windows fall back to a single column.
- **Settings screen** with a **Compact** and a **Full** layout mode. Compact is a small countdown-only window; Full is the landscape layout. The window resizes and the choice is remembered.
- **Author** added to the app (Settings > About), `package.json`, `Cargo.toml` and the bundle metadata (publisher, copyright).
- The main window now starts hidden and is shown once it has the right size.

### 0.1.2
- **Holidays this month.** The home screen lists every holiday in the current month. Finished holidays are italic and greyed out, upcoming ones get an "Upcoming" tag, and today's gets "Today". The status comes from Rust (`monthHolidays`), so the UI does no date math.
- **More holidays are automatic.** Holy Week (Maundy Thursday, Good Friday, Black Saturday) is computed from Easter, and National Heroes Day is the last Monday of August. They work for any year with no proclamation and no JSON entry.
- **JSON overrides generated holidays.** A `holidays.json` entry replaces the generated holiday with the same name in the same year. Maundy Thursday and Good Friday are now listed in the JSON (the computed dates remain as fallback).
- Fixed holiday changes: Dec 31 is now named "New Year's Eve".
- **All Souls' Day (Nov 2) and Christmas Eve (Dec 24) are now fixed-date defaults** every year.
- `holidays.json` is for movable / proclaimed holidays only: Eid'l Fitr, Eid'l Adha, Chinese New Year, Maundy Thursday, Good Friday and special days.
- Compatibility: apps older than 0.1.2 do not compute Black Saturday or National Heroes Day, so they only know them if the JSON lists them. If 0.1.1 installs are still in use, add those entries to `holidays.json` until they upgrade (duplicates are harmless).

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
