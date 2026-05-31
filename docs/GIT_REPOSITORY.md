# Git 仓库与敏感数据

## 远程仓库

| 项 | 值 |
|----|-----|
| 名称 | `origin` |
| URL | `https://github.com/xujian519/yunxi-rust` |
| 默认分支 | `main` |

```bash
# 查看远程
git remote -v

# 首次推送（已有 origin 时）
git push -u origin main

# 更换远程 URL（如需迁移仓库）
git remote set-url origin https://github.com/<org>/<repo>.git
```

## 禁止提交的内容

以下应仅存在于本机或 `~/.yunxi/`，由 `.gitignore` 排除：

| 类别 | 路径示例 |
|------|----------|
| API 密钥 / 数据库密码 | `.yunxi/settings.local.json`、`.yunxi/.env`、`*.env` |
| 会话与运行时状态 | `.yunxi/sessions/`、`.yunxi/vectors/`、`.yunxi/workflows/` |
| Markdown 知识卡片库 | `assets/knowledge-base/**` |
| 语义向量索引 | `assets/knowledge-base/.yunpat-semantic-index.sqlite` |
| 专利知识图谱 | `assets/knowledge_graph/patent_kg.db` |
| 法律法规库 | `assets/knowledge/data/*.db` |
| 构建产物 | `rust/target/`、前端 `node_modules/`、`dist/` |

## 本地配置模板

```bash
cp .yunxi/settings.local.json.example .yunxi/settings.local.json
# 或复制到用户目录
cp .yunxi/settings.local.json.example ~/.yunxi/settings.local.json
# 编辑 env.* 与 semantic.http.apiKey，勿提交 settings.local.json
```

可提交的示例文件：

- `.yunxi/settings.json` — 无密钥的共享默认（模型路由等）
- `.yunxi/settings.json.example`
- `.yunxi/settings.semantic.example.json`
- `.yunxi/settings.vision.example.json`
- `.yunxi/settings.local.json.example`

## 提交前自检

```bash
# 1. 确认无密钥文件在暂存区
git status
git diff --cached --name-only | rg -i 'settings\.local|\.env$|\.sqlite|\.db$' && echo '⚠️ 含敏感路径' || echo '✓ 暂存区无常见敏感路径'

# 2. 确认本地配置已被忽略
git check-ignore -v .yunxi/settings.local.json assets/knowledge-base/

# 3. 若曾误提交密钥，先从索引移除（不删本机文件）
git rm --cached .yunxi/settings.local.json .yunxi/settings.json.bak-* 2>/dev/null || true
```

## 密钥曾进入 Git 历史时

若 `settings.local.json` 等曾被 `git add` 并推送：

1. **立即轮换** 已暴露的 `DEEPSEEK_API_KEY`、`GLM_API_KEY`、`NEO4J_PASSWORD` 等。
2. 从当前分支停止跟踪：`git rm --cached`（见上）。
3. 如需从历史中彻底清除，使用 [git-filter-repo](https://github.com/newren/git-filter-repo) 或 BFG，然后强制推送（需团队协调）。

本仓库检查日期：2026-05-31。曾跟踪文件包括 `.yunxi/settings.local.json` 及其 `.bak`，已从索引移除策略见根目录 `.gitignore`。

## 知识数据获取

大文件不入库，见 [LOCAL_SETUP.md](../LOCAL_SETUP.md) 与 `scripts/sync-knowledge-base.sh`（若存在）在本机同步。
