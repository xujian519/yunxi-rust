# YUNXI.md

本文件为云熙智能体 (YunXi Agent) 在本仓库中工作时提供指导。

## Detected stack
- Languages: Rust.
- Frameworks: none detected from the supported starter markers.

## Verification
- Run Rust verification from the repo root: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`

## Working agreement
- Prefer small, reviewable changes and keep generated bootstrap files aligned with actual repo workflows.
- Keep shared defaults in `.yunxi.json`; reserve `.yunxi/settings.local.json` for machine-local overrides.
- Do not overwrite existing `YUNXI.md` content automatically; update it intentionally when repo workflows change.
