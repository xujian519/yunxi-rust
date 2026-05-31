#!/usr/bin/env bash
# 下载并准备 BGE-M3 ONNX 模型（本地 ONNX 后端，可选；HTTP 8766 服务无需此脚本）
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODEL_DIR="${YUNXI_BGE_MODEL_DIR:-$ROOT/assets/models/bge-m3}"
HF_MODEL="${YUNXI_BGE_HF_ID:-BAAI/bge-m3}"

mkdir -p "$MODEL_DIR"

echo "目标目录: $MODEL_DIR"
echo "HuggingFace 模型: $HF_MODEL"

if command -v optimum-cli >/dev/null 2>&1; then
  optimum-cli export onnx \
    --model "$HF_MODEL" \
    --task feature-extraction \
    "$MODEL_DIR"
elif python3 -c "import optimum" 2>/dev/null; then
  python3 -m optimum.exporters.onnx \
    --model "$HF_MODEL" \
    --task feature-extraction \
    "$MODEL_DIR"
else
  echo "请先安装: pip install optimum[exporters] onnxruntime" >&2
  exit 1
fi

# 统一命名为 model.onnx（部分导出目录可能使用 model.onnx 或 onnx/model.onnx）
if [[ -f "$MODEL_DIR/onnx/model.onnx" && ! -f "$MODEL_DIR/model.onnx" ]]; then
  cp "$MODEL_DIR/onnx/model.onnx" "$MODEL_DIR/model.onnx"
fi

if [[ ! -f "$MODEL_DIR/tokenizer.json" ]]; then
  echo "从 HuggingFace 拉取 tokenizer…"
  python3 - <<PY
from huggingface_hub import snapshot_download
import shutil, os
dst = snapshot_download("$HF_MODEL")
for name in ("tokenizer.json", "tokenizer_config.json", "special_tokens_map.json"):
    src = os.path.join(dst, name)
    if os.path.isfile(src):
        shutil.copy2(src, os.path.join("$MODEL_DIR", name))
PY
fi

echo "完成。启用本地 ONNX：在 .yunxi/settings.json 中设置 semantic.backend=onnx"
