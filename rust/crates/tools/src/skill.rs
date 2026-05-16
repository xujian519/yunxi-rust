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
    let prompt = std::fs::read_to_string(&skill_path).map_err(|error| error.to_string())?;
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

    let mut candidates = Vec::new();
    if let Ok(codex_home) = std::env::var("CODEX_HOME") {
        candidates.push(std::path::PathBuf::from(codex_home).join("skills"));
    }
    candidates.push(std::path::PathBuf::from("/home/bellman/.codex/skills"));

    for root in candidates {
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

pub(crate) fn parse_skill_description(contents: &str) -> Option<String> {
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
