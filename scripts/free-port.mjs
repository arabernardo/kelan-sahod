// Runs before `npm run dev` (via the "predev" script).
// If the dev port is held by a leftover dev server from THIS project, stop it.
// If another program holds it, stop with a clear message instead of killing it.
import { execSync } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const PORTS = [1430, 1431] // keep in sync with vite.config.ts and src-tauri/tauri.conf.json
const projectDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..').toLowerCase()

function run(cmd) {
  try {
    return execSync(cmd, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] })
  } catch {
    return ''
  }
}

function pidsOnPort(port) {
  const pids = new Set()
  if (process.platform === 'win32') {
    for (const line of run('netstat -ano').split(/\r?\n/)) {
      const cols = line.trim().split(/\s+/)
      if (cols[3] === 'LISTENING' && cols[1]?.endsWith(`:${port}`)) pids.add(Number(cols[4]))
    }
  } else {
    for (const pid of run(`lsof -ti tcp:${port} -sTCP:LISTEN`).split(/\s+/)) if (pid) pids.add(Number(pid))
  }
  pids.delete(0)
  return [...pids]
}

function commandLine(pid) {
  if (process.platform === 'win32') {
    return run(
      `powershell -NoProfile -Command "(Get-CimInstance Win32_Process -Filter 'ProcessId=${pid}').CommandLine"`,
    ).trim()
  }
  return run(`ps -o command= -p ${pid}`).trim()
}

let blocked = false
for (const port of PORTS) {
  for (const pid of pidsOnPort(port)) {
    const cmd = commandLine(pid)
    if (cmd.toLowerCase().replaceAll('/', '\\').includes(projectDir.replaceAll('/', '\\'))) {
      console.log(`[free-port] Stopping leftover kelan-sahod dev server on port ${port} (PID ${pid}).`)
      try {
        process.kill(pid)
      } catch {
        run(`taskkill /PID ${pid} /T /F`)
      }
    } else {
      blocked = true
      console.error(`[free-port] Port ${port} is used by another program (PID ${pid}):\n  ${cmd || 'unknown'}`)
    }
  }
}

if (blocked) {
  console.error('[free-port] Close that program, or change the port in vite.config.ts, tauri.conf.json and this script.')
  process.exit(1)
}
