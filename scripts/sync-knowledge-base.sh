#!/usr/bin/env bash
# 从 Obsidian「宝宸知识库」同步知识库到 YunXi，并更新 patent_kg.db 等图谱资产。
#
# 环境变量:
#   YUNXI_OBSIDIAN_VAULT   Obsidian 库路径
#   YUNXI_KB_DEST          Markdown/卡片目标（默认: assets/knowledge-base）
#   YUNXI_YUNPAT_KB        patent_kg 等来源目录（默认: ~/projects/yunpat-agent/knowledge-base）
#   YUNXI_GRAPH_DEST       图谱 DB 目标（默认: assets/knowledge_graph/patent_kg.db）
#
# 用法:
#   ./scripts/sync-knowledge-base.sh [--dry-run] [--skip-graph] [--skip-yunpat-extras]

set -euo pipefail

readonly DEFAULT_VAULT="/Users/xujian/Library/Mobile Documents/iCloud~md~obsidian/Documents/宝宸知识库"
readonly DEFAULT_YUNPAT_KB="${HOME}/projects/yunpat-agent/knowledge-base"
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

VAULT="${YUNXI_OBSIDIAN_VAULT:-${DEFAULT_VAULT}}"
DEST="${YUNXI_KB_DEST:-${REPO_ROOT}/assets/knowledge-base}"
YUNPAT_KB="${YUNXI_YUNPAT_KB:-${DEFAULT_YUNPAT_KB}}"
GRAPH_DEST="${YUNXI_GRAPH_DEST:-${REPO_ROOT}/assets/knowledge_graph/patent_kg.db}"

DRY_RUN=0
SKIP_GRAPH=0
SKIP_YUNPAT_EXTRAS=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      ;;
    --skip-graph)
      SKIP_GRAPH=1
      ;;
    --skip-yunpat-extras)
      SKIP_YUNPAT_EXTRAS=1
      ;;
    -h | --help)
      sed -n '1,12p' "$0"
      exit 0
      ;;
    *)
      echo "未知参数: $1" >&2
      exit 1
      ;;
  esac
  shift
done

if [[ ! -d "${VAULT}" ]]; then
  echo "错误: Obsidian 库不存在: ${VAULT}" >&2
  exit 1
fi

mkdir -p "${DEST}" "$(dirname "${GRAPH_DEST}")"

RSYNC=(rsync -a)
if [[ ${DRY_RUN} -eq 1 ]]; then
  RSYNC=(rsync -anv)
  echo "=== 预览模式（不写入）==="
fi

RSYNC_EXCLUDE=(
  --exclude='.DS_Store'
  --exclude='.obsidian'
  --exclude='.git'
  --exclude='.claude'
  --exclude='.crush'
  --exclude='.omc'
  --exclude='.sisyphus'
  --exclude='.smart-env'
  --exclude='.xiaonuo'
  --exclude='.pytest_cache'
  --exclude='.backup-logs'
  --exclude='patent_kg.db'
  --exclude='patent_kg.db-shm'
  --exclude='patent_kg.db-wal'
)

log_section() {
  echo
  echo "== $1 =="
}

sync_path() {
  local src="$1"
  local dst="$2"
  if [[ ! -e "${src}" ]]; then
    echo "跳过（源不存在）: ${src}"
    return 0
  fi
  echo "→ ${dst#${REPO_ROOT}/}"
  "${RSYNC[@]}" "${RSYNC_EXCLUDE[@]}" "${src}" "${dst}"
}

remap_semantic_index_paths() {
  local index_file="${DEST}/.yunpat-semantic-index.sqlite"
  if [[ ! -f "${index_file}" ]]; then
    echo "跳过语义索引路径映射（文件不存在）"
    return 0
  fi
  if [[ ${DRY_RUN} -eq 1 ]]; then
    echo "预览: 将 remap ${index_file} 内 Obsidian 路径 → ${DEST}"
    return 0
  fi
  if ! command -v sqlite3 >/dev/null 2>&1; then
    echo "警告: 未安装 sqlite3，跳过语义索引路径映射" >&2
    return 0
  fi

  sqlite3 "${index_file}" <<SQL
UPDATE chunks
SET file_path = REPLACE(file_path, '${VAULT}/Wiki/', '${DEST}/')
WHERE file_path LIKE '${VAULT}/Wiki/%';

UPDATE chunks
SET file_path = REPLACE(file_path, '${VAULT}/', '${DEST}/')
WHERE file_path LIKE '${VAULT}/%' AND file_path NOT LIKE '${DEST}/%';
SQL

  local chunks mapped model
  chunks="$(sqlite3 "${index_file}" "SELECT COUNT(*) FROM chunks;")"
  mapped="$(sqlite3 "${index_file}" "SELECT COUNT(*) FROM chunks WHERE file_path LIKE '${DEST}/%';")"
  model="$(sqlite3 "${index_file}" "SELECT value FROM index_meta WHERE key='embedding_model';" 2>/dev/null || true)"
  echo "语义索引路径已映射: chunks=${chunks}, 指向 YunXi=${mapped}, model=${model:-unknown}"
}

copy_patent_kg() {
  local src="${YUNPAT_KB}/patent_kg.db"
  if [[ ! -f "${src}" ]]; then
    echo "跳过 patent_kg.db（源不存在: ${src}）"
    return 0
  fi

  local kb_dest="${DEST}/patent_kg.db"
  echo "→ ${GRAPH_DEST#${REPO_ROOT}/}"
  echo "→ ${kb_dest#${REPO_ROOT}/}"

  if [[ ${DRY_RUN} -eq 1 ]]; then
    echo "预览: cp ${src} → ${GRAPH_DEST} 与 ${kb_dest}"
    return 0
  fi

  for target in "${GRAPH_DEST}" "${kb_dest}"; do
    if [[ -f "${target}" ]]; then
      cp -p "${target}" "${target}.bak.$(date +%Y%m%d%H%M%S)"
    fi
    cp -p "${src}" "${target}"
    rm -f "${target}-wal" "${target}-shm" 2>/dev/null || true
  done
  echo "patent_kg.db 已同步（来源: ${src}）"
}

log_section "Obsidian → YunXi"
echo "源: ${VAULT}"
echo "目标: ${DEST}"

sync_path "${VAULT}/cards/" "${DEST}/"
sync_path "${VAULT}/card-index.json" "${DEST}/"
sync_path "${VAULT}/.yunpat-semantic-index.sqlite" "${DEST}/"

for name in \
  All-Concepts.md \
  All-Concepts-拆分-01-授权条件.md \
  All-Concepts-拆分-02-程序问题.md \
  "All-Concepts-拆分-03-扩展概念(新增60个概念)-职务发明.md" \
  Concept-Hierarchy.md \
  Concept-Index.md; do
  sync_path "${VAULT}/Wiki/${name}" "${DEST}/"
done

for kb in 专利实务 审查指南 法律法规 专利侵权 专利判决 复审无效 书籍 个人笔记; do
  sync_path "${VAULT}/Wiki/${kb}/" "${DEST}/${kb}/"
done
sync_path "${VAULT}/方法论/" "${DEST}/方法论/"

log_section "语义索引路径映射"
remap_semantic_index_paths

if [[ ${SKIP_GRAPH} -eq 0 ]]; then
  log_section "patent_kg.db（yunpat-agent → YunXi）"
  echo "来源目录: ${YUNPAT_KB}"
  copy_patent_kg
else
  echo "跳过 patent_kg.db（--skip-graph）"
fi

if [[ ${SKIP_YUNPAT_EXTRAS} -eq 0 && -d "${YUNPAT_KB}" ]]; then
  log_section "yunpat-agent 附加资产"
  for extra in ipc-classification legal-system; do
    sync_path "${YUNPAT_KB}/${extra}/" "${DEST}/${extra}/"
  done
elif [[ ${SKIP_YUNPAT_EXTRAS} -eq 1 ]]; then
  echo "跳过 yunpat 附加目录（--skip-yunpat-extras）"
else
  echo "跳过 yunpat 附加目录（来源不存在: ${YUNPAT_KB}）"
fi

echo
if [[ ${DRY_RUN} -eq 1 ]]; then
  echo "预览完成。去掉 --dry-run 后执行实际同步。"
else
  echo "同步完成: $(date '+%Y-%m-%d %H:%M:%S')"
fi

cat <<NOTE

提示:
  • Obsidian 更新 Wiki 后，可在库内运行 \`make index\` 重建语义索引，再执行本脚本。
  • patent_kg.db 由 yunpat-agent 维护；Obsidian 库内无此文件。
  • 安装定时任务: ./scripts/install-kb-sync-launchd.sh install

NOTE
