// Starts `tauri dev` on the first free port, so "port already in use" never blocks you.
//
//   npm run tauri:dev                          (extra flags are passed on, e.g. npm run tauri:dev -- --release)
//   npm run tauri:dev -- --date=2026-11-27     (pretend today is that date, e.g. to see the payday screen)
//   npm run tauri:dev -- --print               (just show the chosen port and settings, don't start anything)
//
// Why a script: Tauri reads the dev server address from tauri.conf.json, so Vite can't just
// pick another port by itself. This script finds a free port, then tells BOTH Vite
// (--port, --strictPort) and Tauri (devUrl) to use it. Nothing in your config files changes.
// Plain `npm run tauri dev` still works and still uses port 1420.

import net from 'node:net'
import { spawn } from 'node:child_process'
import { createRequire } from 'node:module'

const START_PORT = 1420
const MAX_TRIES = 50

/** Can we listen on this port on this address? Addresses the machine doesn't have don't count. */
function canListen(port, host) {
  return new Promise((resolve) => {
    const server = net.createServer()
    server.once('error', (err) => resolve(err.code === 'EADDRNOTAVAIL' || err.code === 'EAFNOSUPPORT'))
    server.once('listening', () => server.close(() => resolve(true)))
    server.listen(port, host)
  })
}

// Vite's "localhost" can be IPv4 or IPv6, so the port must be free on both.
async function isFree(port) {
  return (await canListen(port, '127.0.0.1')) && (await canListen(port, '::1'))
}

async function findFreePort(start = START_PORT) {
  for (let port = start; port < start + MAX_TRIES; port++) {
    if (await isFree(port)) return port
  }
  throw new Error(`No free port between ${start} and ${start + MAX_TRIES - 1}`)
}

function buildConfig(port) {
  return {
    build: {
      devUrl: `http://localhost:${port}`,
      beforeDevCommand: `npm run dev -- --port ${port} --strictPort`,
    },
  }
}

// --date 2026-11-27  or  --date=2026-11-27  (removed from the args that go on to Tauri)
function takeDateArg(args) {
  const rest = []
  let date
  for (let i = 0; i < args.length; i++) {
    const arg = args[i]
    if (arg === '--date') date = args[++i] ?? ''
    else if (arg.startsWith('--date=')) date = arg.slice('--date='.length)
    else rest.push(arg)
  }
  return { date, rest }
}

// A real calendar date written as YYYY-MM-DD ("2026-02-30" is rejected).
function isRealDate(text) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(text)) return false
  const parsed = new Date(`${text}T00:00:00Z`)
  return !Number.isNaN(parsed.getTime()) && parsed.toISOString().slice(0, 10) === text
}

async function main() {
  const { date, rest: args } = takeDateArg(process.argv.slice(2))
  if (date !== undefined && !isRealDate(date)) {
    throw new Error(`--date must be a real date written as YYYY-MM-DD (for example --date=2026-11-27). You gave: "${date}"`)
  }
  const port = await findFreePort()
  const config = buildConfig(port)

  if (port !== START_PORT) {
    console.log(`Port ${START_PORT} is in use, using ${port} instead.`)
  }

  // The app reads this in debug builds only (see README: "Test a specific date").
  const env = date ? { ...process.env, KELAN_SAHOD_TODAY: date } : process.env
  if (date) console.log(`Test date: the app will pretend today is ${date}.`)

  if (args.includes('--print')) {
    console.log(JSON.stringify({ port, testDate: date ?? null, config }, null, 2))
    return
  }

  const tauriCli = createRequire(import.meta.url).resolve('@tauri-apps/cli/tauri.js')
  const child = spawn(process.execPath, [tauriCli, 'dev', '--config', JSON.stringify(config), ...args], {
    stdio: 'inherit',
    env,
  })
  child.on('exit', (code) => process.exit(code ?? 0))
}

main().catch((err) => {
  console.error(err.message)
  process.exit(1)
})
