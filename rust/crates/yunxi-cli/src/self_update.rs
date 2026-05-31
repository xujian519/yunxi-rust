//! 本机自更新：在仓库内重新编译并安装 `yunxi` 二进制。

use std::path::PathBuf;
use std::process::Command;

pub fn run_self_update() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = locate_repo_root().ok_or(
        "未找到 rust/Cargo.toml；请在 YunXi 仓库根目录运行，或手动执行: cd rust && cargo build --release",
    )?;

    let rust_dir = repo_root.join("rust");
    let target_bin = rust_dir.join("target/release/yunxi");
    let current = std::env::current_exe()?;

    println!("云熙智能体 — 本机自更新\n");
    println!("仓库: {}", repo_root.display());
    println!("当前二进制: {}", current.display());
    println!("\n正在编译 release 版本…\n");

    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "yunxi-cli"])
        .current_dir(&rust_dir)
        .status()?;

    if !status.success() {
        return Err("cargo build --release 失败".into());
    }

    if !target_bin.is_file() {
        return Err(format!("编译产物不存在: {}", target_bin.display()).into());
    }

    if current != target_bin {
        std::fs::copy(&target_bin, &current)?;
        println!("已更新: {}", current.display());
    } else {
        println!(
            "当前正在使用仓库内二进制，编译完成即已生效: {}",
            target_bin.display()
        );
    }

    let version = Command::new(&current).arg("--version").output()?;
    if version.status.success() {
        println!("{}", String::from_utf8_lossy(&version.stdout).trim());
    }

    Ok(())
}

fn locate_repo_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    for ancestor in cwd.ancestors() {
        if ancestor.join("rust/Cargo.toml").is_file() {
            return Some(ancestor.to_path_buf());
        }
    }
    std::env::current_exe().ok().and_then(|exe| {
        exe.parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .filter(|root| root.join("rust/Cargo.toml").is_file())
    })
}
