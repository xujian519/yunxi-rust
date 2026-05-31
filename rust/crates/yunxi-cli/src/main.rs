fn main() {
    if let Err(error) = yunxi_cli::run_cli() {
        eprintln!(
            "error: {error}

运行 `yunxi --help` 查看用法。"
        );
        std::process::exit(1);
    }
}
