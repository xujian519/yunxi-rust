#!/usr/bin/env bash
# 安装 / 卸载 YunXi 知识库定时同步（macOS launchd，默认每天 03:15）
#
# Cron 替代（Linux 或不想用 launchd 时，每天 03:15）:
#   crontab -e
#   15 3 * * * /bin/bash /Users/xujian/projects/YunXi/scripts/sync-knowledge-base.sh >> ~/Library/Logs/YunXi/kb-sync.log 2>&1
#
#   ./scripts/install-kb-sync-launchd.sh uninstall
#   ./scripts/install-kb-sync-launchd.sh status
#   ./scripts/install-kb-sync-launchd.sh run-now

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly LABEL="com.yunxi.kb-sync"
readonly PLIST_SRC="${SCRIPT_DIR}/com.yunxi.kb-sync.plist"
readonly PLIST_DST="${HOME}/Library/LaunchAgents/${LABEL}.plist"
readonly LOG_DIR="${HOME}/Library/Logs/YunXi"

usage() {
  sed -n '1,8p' "$0"
}

render_plist() {
  sed \
    -e "s|__YUNXI_REPO__|${REPO_ROOT}|g" \
    -e "s|__HOME__|${HOME}|g" \
    "${PLIST_SRC}"
}

cmd_install() {
  mkdir -p "${LOG_DIR}" "${HOME}/Library/LaunchAgents"
  render_plist >"${PLIST_DST}"
  chmod 644 "${PLIST_DST}"
  launchctl bootout "gui/${UID}/${LABEL}" 2>/dev/null || true
  launchctl bootstrap "gui/${UID}" "${PLIST_DST}"
  echo "已安装: ${PLIST_DST}"
  echo "计划: 每天 03:15 执行 ${REPO_ROOT}/scripts/sync-knowledge-base.sh"
  echo "日志: ${LOG_DIR}/kb-sync.log"
}

cmd_uninstall() {
  launchctl bootout "gui/${UID}/${LABEL}" 2>/dev/null || true
  rm -f "${PLIST_DST}"
  echo "已卸载 ${LABEL}"
}

cmd_status() {
  if [[ -f "${PLIST_DST}" ]]; then
    echo "plist: ${PLIST_DST}"
    plutil -p "${PLIST_DST}" | head -20
  else
    echo "未安装（${PLIST_DST} 不存在）"
  fi
  echo
  launchctl print "gui/${UID}/${LABEL}" 2>/dev/null | head -20 || echo "launchd 任务未加载"
}

cmd_run_now() {
  echo "立即执行同步..."
  "${REPO_ROOT}/scripts/sync-knowledge-base.sh"
}

case "${1:-}" in
  install) cmd_install ;;
  uninstall) cmd_uninstall ;;
  status) cmd_status ;;
  run-now) cmd_run_now ;;
  -h | --help) usage ;;
  *)
    usage >&2
    exit 1
    ;;
esac
