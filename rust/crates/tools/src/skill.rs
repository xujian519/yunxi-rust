use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct SkillInput {
    pub skill: String,
    pub args: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SkillOutput {
    pub skill: String,
    pub path: String,
    pub args: Option<String>,
    pub description: Option<String>,
    pub prompt: String,
}

pub(crate) fn execute_skill(input: SkillInput) -> Result<SkillOutput, String> {
    let skill_path = resolve_skill_path(&input.skill)?;
    let raw_prompt = std::fs::read_to_string(&skill_path).map_err(|error| error.to_string())?;

    let prompt = if raw_prompt.trim_start().starts_with("<?xml")
        || raw_prompt.trim_start().starts_with("<skill")
    {
        let skills_dir = skill_path
            .parent()
            .ok_or_else(|| String::from("invalid skill path"))?;
        resolve_includes(&raw_prompt, skills_dir)?
    } else {
        raw_prompt
    };

    let description = parse_skill_description(&prompt);

    Ok(SkillOutput {
        skill: input.skill,
        path: skill_path.display().to_string(),
        args: input.args,
        description,
        prompt,
    })
}

pub(crate) fn resolve_skill_path(skill: &str) -> Result<std::path::PathBuf, String> {
    let requested = skill.trim().trim_start_matches('/').trim_start_matches('$');
    if requested.is_empty() {
        return Err(String::from("skill must not be empty"));
    }

    for root in build_skill_search_dirs() {
        let direct = root.join(requested).join("SKILL.md");
        if direct.exists() {
            return Ok(direct);
        }

        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path().join("SKILL.md");
                if !path.exists() {
                    continue;
                }
                if entry
                    .file_name()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(requested)
                {
                    return Ok(path);
                }
            }
        }
    }

    Err(format!("unknown skill: {requested}"))
}

fn build_skill_search_dirs() -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;
    let mut dirs = Vec::new();

    // 1. 项目内置 skills（编译时确定）
    let builtin = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../assets/skills");
    if builtin.is_dir() {
        dirs.push(builtin);
    }

    // 2. 用户主目录
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        dirs.push(PathBuf::from(home).join(".yunxi/skills"));
    }

    // 3. 环境变量覆盖
    if let Ok(skills_dir) = std::env::var("YUNXI_SKILLS_DIR") {
        dirs.push(PathBuf::from(skills_dir));
    }

    // 4. 兼容旧路径
    if let Some(home) = runtime::env_var("YUNXI_HOME") {
        let legacy = PathBuf::from(&home).join("skills");
        if legacy.is_dir() {
            dirs.push(legacy);
        }
    }

    dirs
}

pub(crate) fn parse_skill_description(contents: &str) -> Option<String> {
    // 优先解析 YAML frontmatter
    if let Some(frontmatter) = extract_yaml_frontmatter(contents) {
        for line in frontmatter.lines() {
            if let Some(value) = line.strip_prefix("description:") {
                let trimmed = value.trim().trim_matches('"').trim_matches('\'');
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    // 回退: 在全文中搜索 description: 行
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("description:") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

const MAX_INCLUDE_DEPTH: usize = 3;

pub(crate) fn resolve_includes(
    content: &str,
    base_dir: &std::path::Path,
) -> Result<String, String> {
    let mut resolved = content.to_string();
    for _depth in 0..MAX_INCLUDE_DEPTH {
        let mut replaced = false;
        let mut result = String::with_capacity(resolved.len());
        let mut remaining = resolved.as_str();

        while let Some(tag_start) = remaining.find("<include ") {
            result.push_str(&remaining[..tag_start]);
            let after_start = &remaining[tag_start..];
            match parse_include_tag(after_start, base_dir) {
                Ok((module_content, consumed)) => {
                    result.push_str(&module_content);
                    remaining = &after_start[consumed..];
                    replaced = true;
                }
                Err(e) => return Err(e),
            }
        }
        result.push_str(remaining);
        resolved = result;
        if !replaced {
            break;
        }
    }
    Ok(resolved)
}

fn parse_include_tag(input: &str, base_dir: &std::path::Path) -> Result<(String, usize), String> {
    let ref_start = input
        .find("ref=\"")
        .ok_or_else(|| String::from("invalid include tag: missing ref attribute"))?;
    let ref_val_start = ref_start + 5;
    let ref_val_end = input[ref_val_start..]
        .find('"')
        .map(|p| ref_val_start + p)
        .ok_or_else(|| String::from("invalid include tag: unclosed ref attribute"))?;
    let ref_value = &input[ref_val_start..ref_val_end];

    let tag_end = input[ref_val_end..]
        .find('>')
        .map(|p| ref_val_end + p)
        .ok_or_else(|| String::from("invalid include tag: missing closing bracket"))?;

    let module_path = base_dir.join(ref_value).with_extension("xml");
    let module_content = std::fs::read_to_string(&module_path)
        .map_err(|e| format!("failed to read include '{}': {e}", module_path.display()))?;

    Ok((module_content, tag_end + 1))
}

fn extract_yaml_frontmatter(contents: &str) -> Option<&str> {
    let text = contents.strip_prefix("---")?;
    let end = text.find("---")?;
    Some(&text[..end])
}
