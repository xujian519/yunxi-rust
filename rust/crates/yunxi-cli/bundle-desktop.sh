#!/usr/bin/env bash
# 从任意目录调用均可；打包 macOS .app 并打开
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
FRONTEND="$ROOT/frontend"
APP="$ROOT/../../target/release/bundle/macos/云熙智能体.app"

echo "→ 前端构建 + Tauri 打包…"
(cd "$FRONTEND" && npm run bundle:desktop)

if [[ ! -d "$APP" ]]; then
  echo "未找到 .app，请检查: $APP" >&2
  exit 1
fi

echo "→ 打开 $APP"
open "$APP"
