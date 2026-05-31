# reference_data

本目录存放**移植对照快照**（命令、工具、子系统路径清单），供 `compat-harness` 与 Python  parity 审计使用。

- JSON 中的 `source_hint` 等字段记录**已归档的上游 TypeScript 树**中的原始路径，仅用于差异审计，不代表云熙运行时仍依赖这些名称。
- 若需重新生成快照，请使用 `compat-harness` 或 `src/parity_audit.py`，并将归档根目录设为 `archive/upstream_ts_snapshot/`。

云熙运行时配置与文档请使用 `.yunxi/`、`.yunxi.json`、`YUNXI.md`。
