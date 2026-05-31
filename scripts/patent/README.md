# 专利案件办公格式脚本

## MarkItDown（推荐用于多格式统一转 Markdown）

```bash
pip install 'markitdown[pdf,docx,pptx]'
python3 scripts/patent/markitdown_convert.py /path/to/file.pdf
```

环境变量：

- `YUNXI_MARKITDOWN_SCRIPT` — 脚本路径（默认 `scripts/patent/markitdown_convert.py`）
- `YUNXI_MARKITDOWN_PYTHON` — Python 解释器（默认 `python3`）

## 图片 OCR — oMLX 多模态（推荐，:8009）

与 **BGE-M3 嵌入** 共用本机 oMLX 服务（默认 `http://127.0.0.1:8009`）：

| 能力 | 端点 | 模型示例 |
|------|------|----------|
| 向量嵌入 | `POST /v1/embeddings` | `bge-m3-mlx-8bit` |
| 图片文字识别 | `POST /v1/chat/completions` | `gemma-4-e2b-it-4bit`（默认，与 oMLX `default_model` 一致） |

工具：

- **`VisionOcr`** — 仅多模态
- **`LocalOcr`** — `backend: auto` 时先 Vision，失败再 Tesseract

配置示例：复制 `.yunxi/settings.vision.example.json` 到 `~/.yunxi/settings.local.json`。**只需在 `semantic.http` 写一次 `apiKey`**，`vision` 自动共用。

环境变量：

- `OMLX_API_KEY` / `EMBEDDING_API_KEY` — 嵌入与多模态共用
- `YUNXI_VISION_MODEL` — 覆盖多模态模型名
- `YUNXI_VISION_URL` — 覆盖 base URL（默认与 `semantic.http.baseUrl` 一致）

## 扫描 PDF 分页 OCR

```bash
brew install poppler   # 提供 pdftoppm
```

- `/extract` 或 `PdfParse` + `operation: scanned_ocr`
- 文本层过少时 `/extract` 自动改用分页 OCR（默认最多 24 页，`YUNXI_PDF_OCR_MAX_PAGES`）
- 每页：`pdftoppm` → PNG → oMLX `gemma-4-e2b-it-4bit`（失败则 Tesseract）

环境变量：`YUNXI_PDF_OCR_DPI`（默认 150）、`YUNXI_PDFTOPPM_CMD`

## Tesseract 回退（离线）

```bash
brew install tesseract tesseract-lang
tesseract image.png stdout -l chi_sim+eng
```

- `YUNXI_TESSERACT_CMD`、`YUNXI_OCR_LANG`
- `LocalOcr` 的 `backend: tesseract` 强制仅用 Tesseract
