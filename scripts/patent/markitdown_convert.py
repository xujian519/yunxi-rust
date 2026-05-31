#!/usr/bin/env python3
"""将办公文档转为 Markdown（需 pip install 'markitdown[pdf,docx,pptx]'）。"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) < 2:
        print(json.dumps({"error": "usage: markitdown_convert.py <file_path>"}), file=sys.stderr)
        return 2

    path = Path(sys.argv[1]).expanduser().resolve()
    if not path.is_file():
        print(json.dumps({"error": f"file not found: {path}"}), file=sys.stderr)
        return 1

    try:
        from markitdown import MarkItDown
    except ImportError:
        print(
            json.dumps(
                {
                    "error": "markitdown not installed",
                    "hint": "pip install 'markitdown[pdf,docx,pptx]'",
                }
            ),
            file=sys.stderr,
        )
        return 1

    try:
        md = MarkItDown()
        result = md.convert(str(path))
        text = result.text_content or ""
        print(
            json.dumps(
                {
                    "file_path": str(path),
                    "engine": "markitdown",
                    "markdown": text,
                    "char_count": len(text),
                },
                ensure_ascii=False,
            )
        )
        return 0
    except Exception as exc:  # noqa: BLE001
        print(json.dumps({"error": str(exc), "file_path": str(path)}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
