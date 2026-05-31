//! 云熙环境变量读取辅助。

/// 读取环境变量（忽略空值）。
#[must_use]
pub fn env_var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

/// 从映射中读取非空字符串。
#[must_use]
pub fn map_get(map: &std::collections::BTreeMap<String, String>, key: &str) -> Option<String> {
    map.get(key).filter(|v| !v.trim().is_empty()).cloned()
}
