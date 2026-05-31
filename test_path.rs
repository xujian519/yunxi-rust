#!/usr/bin/env rust-script

use std::path::Path;

fn main() {
    // 检查规则文件路径
    let paths = vec![
        "rust/crates/constitutional-engine/assets/constitutional",
        "crates/constitutional-engine/assets/constitutional",
        "assets/constitutional",
    ];

    for path in &paths {
        let p = Path::new(path);
        println!("检查路径: {}", path);
        if p.exists() {
            println!("  ✓ 路径存在");
            if p.is_dir() {
                println!("  ✓ 是目录");
                let entries = std::fs::read_dir(p).unwrap();
                for entry in entries {
                    let entry = entry.unwrap();
                    println!("    - {}", entry.file_name().to_string_lossy());
                }
            }
        } else {
            println!("  ✗ 路径不存在");
        }
        println!();
    }
}