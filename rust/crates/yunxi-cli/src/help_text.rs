use std::io::{self, Write};

use commands::{render_full_slash_help, resume_supported_slash_commands};

use crate::VERSION;

pub(crate) fn print_help_to(out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "云熙智能体 (YunXi Agent) v{VERSION}")?;
    writeln!(out)?;
    writeln!(out, "用法：")?;
    writeln!(
        out,
        "  yunxi [--model MODEL] [--allowedTools TOOL[,TOOL...]]"
    )?;
    writeln!(out, "      启动交互式 REPL")?;
    writeln!(
        out,
        "  yunxi [--model MODEL] [--output-format text|json] prompt TEXT"
    )?;
    writeln!(out, "      发送单次提示并退出")?;
    writeln!(
        out,
        "  yunxi [--model MODEL] [--output-format text|json] TEXT"
    )?;
    writeln!(out, "      简写模式的非交互式提示")?;
    writeln!(
        out,
        "  yunxi --resume SESSION.json [/status] [/compact] [...]"
    )?;
    writeln!(out, "      检查或维护已保存的会话（不进入 REPL）")?;
    writeln!(out, "  yunxi dump-manifests")?;
    writeln!(out, "  yunxi bootstrap-plan")?;
    writeln!(
        out,
        "  yunxi system-prompt [--cwd PATH] [--date YYYY-MM-DD]"
    )?;
    writeln!(out, "  yunxi login")?;
    writeln!(out, "  yunxi logout")?;
    writeln!(out, "  yunxi init")?;
    writeln!(out, "  yunxi doctor")?;
    writeln!(out, "  yunxi config")?;
    writeln!(out, "      查看配置状态与推荐下一步")?;
    writeln!(out, "  yunxi config init [--user|--project] [-i]")?;
    writeln!(
        out,
        "      从模板创建 settings.local.json（-i 交互写入 API 密钥）"
    )?;
    writeln!(out, "  yunxi config show [env|hooks|model]")?;
    writeln!(out, "      查看合并后的配置详情")?;
    writeln!(out, "  yunxi server [--host HOST] [--port PORT]")?;
    writeln!(
        out,
        "      启动本地 HTTP/WebSocket API（默认 127.0.0.1:8765）"
    )?;
    writeln!(out, "  yunxi self-update")?;
    writeln!(out, "      在仓库内重新编译并更新当前 yunxi 二进制")?;
    writeln!(out, "  yunxi-desktop")?;
    writeln!(
        out,
        "      专利案件桌面客户端（Tauri + React，检索/对比/审查/撰写）"
    )?;
    writeln!(out)?;
    writeln!(out, "参数：")?;
    writeln!(out, "  --model MODEL              覆盖当前模型")?;
    writeln!(
        out,
        "  --output-format FORMAT     非交互式输出格式：text 或 json"
    )?;
    writeln!(
        out,
        "  --permission-mode MODE     设置权限模式：read-only、workspace-write 或 danger-full-access"
    )?;
    writeln!(out, "  --dangerously-skip-permissions  跳过所有权限检查")?;
    writeln!(
        out,
        "  --allowedTools TOOLS       限制启用的工具（可重复；支持逗号分隔的别名）"
    )?;
    writeln!(out, "  --version, -V              显示版本和构建信息")?;
    writeln!(
        out,
        "  --profile NAME             保留参数（patent 专屏已移除，请用 yunxi-desktop）"
    )?;
    writeln!(out)?;
    writeln!(out, "交互式斜杠命令：")?;
    writeln!(out, "{}", render_full_slash_help())?;
    writeln!(out)?;
    let resume_commands = resume_supported_slash_commands()
        .into_iter()
        .map(|spec| match spec.argument_hint {
            Some(argument_hint) => format!("/{} {}", spec.name, argument_hint),
            None => format!("/{}", spec.name),
        })
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(out, "支持恢复的命令：{resume_commands}")?;
    writeln!(out, "示例：")?;
    writeln!(out, "  yunxi --model claude-opus \"总结这个仓库\"")?;
    writeln!(
        out,
        "  yunxi --output-format json prompt \"解释 src/main.rs\""
    )?;
    writeln!(out, "  yunxi --allowedTools read,glob \"总结 Cargo.toml\"")?;
    writeln!(
        out,
        "  yunxi --resume session.json /status /diff /export notes.txt"
    )?;
    writeln!(out, "  yunxi login")?;
    writeln!(out, "  yunxi init")?;
    writeln!(out, "  yunxi doctor")?;
    writeln!(out, "  yunxi config init -i")?;
    Ok(())
}

pub(crate) fn print_help() {
    let _ = print_help_to(&mut io::stdout());
}
