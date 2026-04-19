#!/usr/bin/env bash
# Pre-launch guard for `bun run tauri:dev`.
# Kills stale Vibo dev processes and ensures port 8080 is free so strictPort
# in vite.config.ts never has to fail. Safe to run repeatedly.
set -u

echo "[prelaunch] clearing stale dev processes..."

# App binary spawned by `tauri dev` (debug profile).
pkill -f "target/debug/app" 2>/dev/null || true
# Tauri CLI wrapper.
pkill -f "tauri dev" 2>/dev/null || true
# Vite dev server for this repo.
pkill -f "vite.*zettel-spark-flow" 2>/dev/null || true

# Free port 8080 regardless of who owns it.
PIDS=$(lsof -ti tcp:8080 2>/dev/null || true)
if [ -n "${PIDS}" ]; then
  echo "[prelaunch] killing processes on :8080 -> ${PIDS}"
  kill -9 ${PIDS} 2>/dev/null || true
fi

# Tiny grace period for sockets to release.
sleep 0.3
echo "[prelaunch] ready."
exit 0
