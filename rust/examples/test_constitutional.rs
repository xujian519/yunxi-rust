fn main() {
    match tools::ConstitutionalCheckTool::new() {
        Ok(_) => println!("✅ 工具创建成功"),
        Err(e) => println!("❌ 工具创建失败: {}", e),
    }
}