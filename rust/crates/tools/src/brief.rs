use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config_tool::iso8601_timestamp;

#[derive(Debug, Deserialize)]
pub(crate) struct BriefInput {
    pub message: String,
    pub attachments: Option<Vec<String>>,
    pub status: BriefStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum BriefStatus {
    Normal,
    Proactive,
}

#[derive(Debug, Serialize)]
pub(crate) struct BriefOutput {
    pub message: String,
    pub attachments: Option<Vec<ResolvedAttachment>>,
    #[serde(rename = "sentAt")]
    pub sent_at: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ResolvedAttachment {
    pub path: String,
    pub size: u64,
    #[serde(rename = "isImage")]
    pub is_image: bool,
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn execute_sleep(input: SleepInput) -> SleepOutput {
    std::thread::sleep(std::time::Duration::from_millis(input.duration_ms));
    SleepOutput {
        duration_ms: input.duration_ms,
        message: format!("Slept for {}ms", input.duration_ms),
    }
}

pub(crate) fn execute_brief(input: BriefInput) -> Result<BriefOutput, String> {
    if input.message.trim().is_empty() {
        return Err(String::from("message must not be empty"));
    }

    let attachments = input
        .attachments
        .as_ref()
        .map(|paths| {
            paths
                .iter()
                .map(|path| resolve_attachment(path))
                .collect::<Result<Vec<_>, String>>()
        })
        .transpose()?;

    let message = match input.status {
        BriefStatus::Normal | BriefStatus::Proactive => input.message,
    };

    Ok(BriefOutput {
        message,
        attachments,
        sent_at: iso8601_timestamp(),
    })
}

fn resolve_attachment(path: &str) -> Result<ResolvedAttachment, String> {
    let resolved = std::fs::canonicalize(path).map_err(|error| error.to_string())?;
    let metadata = std::fs::metadata(&resolved).map_err(|error| error.to_string())?;
    Ok(ResolvedAttachment {
        path: resolved.display().to_string(),
        size: metadata.len(),
        is_image: is_image_path(&resolved),
    })
}

fn is_image_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg")
    )
}

// SleepInput/SleepOutput are defined here since they are small and related
#[derive(Debug, serde::Deserialize)]
pub(crate) struct SleepInput {
    pub duration_ms: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct SleepOutput {
    pub duration_ms: u64,
    pub message: String,
}
