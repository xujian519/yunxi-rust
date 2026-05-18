use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::SlashCommand;

// ============================================================================
// Patent Intent
// ============================================================================

/// Patent-specific intent variants recognized from natural language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatentIntent {
    /// Draft or prepare patent application documents.
    Draft,
    /// Respond to an office action (examination opinion).
    OAResponse,
    /// Search for prior art or patentability analysis.
    Search,
    /// Evaluate patent quality / innovation height.
    QualityCheck,
    /// Check formal requirements and formatting.
    FormalityCheck,
    /// Patent portfolio or filing strategy.
    Strategy,
    /// Compare patents or technical features.
    Compare,
}

// ============================================================================
// Intent
// ============================================================================

/// Recognized user intent from natural language input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// Maps to a slash command.
    Command(SlashCommand),
    /// Patent workflow intent.
    Patent(PatentIntent),
    /// Plain chat message — no command detected.
    Chat,
}

// ============================================================================
// Legal Scenario Types
// ============================================================================

/// Business domain for legal scenario identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Patent,
    Trademark,
    Legal,
    Copyright,
    Other,
}

impl Domain {
    /// Return the domain as a static string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Patent => "patent",
            Self::Trademark => "trademark",
            Self::Legal => "legal",
            Self::Copyright => "copyright",
            Self::Other => "other",
        }
    }
}

/// Task type for legal scenario identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    CreativityAnalysis,
    NoveltyAnalysis,
    Infringement,
    Similarity,
    Validity,
    Drafting,
    Search,
    Other,
}

impl TaskType {
    /// Return the task type as a static string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreativityAnalysis => "creativity_analysis",
            Self::NoveltyAnalysis => "novelty_analysis",
            Self::Infringement => "infringement",
            Self::Similarity => "similarity",
            Self::Validity => "validity",
            Self::Drafting => "drafting",
            Self::Search => "search",
            Self::Other => "other",
        }
    }
}

/// Phase of a legal matter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Application,
    Examination,
    Opposition,
    Litigation,
    Other,
}

impl Phase {
    /// Return the phase as a static string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Examination => "examination",
            Self::Opposition => "opposition",
            Self::Litigation => "litigation",
            Self::Other => "other",
        }
    }
}

/// Result of legal scenario identification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioContext {
    /// Identified domain.
    pub domain: Domain,
    /// Identified task type.
    pub task_type: TaskType,
    /// Identified phase.
    pub phase: Phase,
    /// Confidence score in the range [0.0, 1.0].
    pub confidence: f64,
    /// Variables extracted from the input text.
    #[serde(default)]
    pub extracted_variables: HashMap<String, String>,
    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl ScenarioContext {
    /// Suggest an agent ID based on the scenario.
    #[must_use]
    pub fn suggested_agent_id(&self) -> Option<&'static str> {
        if self.confidence < 0.15 {
            return None;
        }
        match (self.domain, self.task_type) {
            (Domain::Patent, TaskType::CreativityAnalysis) => Some("innovation-evaluator"),
            (Domain::Patent, TaskType::NoveltyAnalysis) => Some("patent-analyzer"),
            (Domain::Patent, TaskType::Infringement) => Some("patent-infringement-analyzer"),
            (Domain::Patent, TaskType::Validity) => Some("invalidation"),
            (Domain::Patent, TaskType::Drafting) => Some("claim-generator"),
            (Domain::Patent, TaskType::Search) => Some("search"),
            (Domain::Patent, TaskType::Other) => {
                if self.phase == Phase::Examination {
                    Some("patent-responder")
                } else {
                    None
                }
            }
            (Domain::Trademark, TaskType::Infringement | TaskType::Similarity) => {
                Some("trademark-analyzer")
            }
            (Domain::Legal, _) => Some("legal-qa"),
            _ => None,
        }
    }

    /// Return the legal search domain for this scenario.
    #[must_use]
    pub fn legal_search_domain(&self) -> Option<&'static str> {
        match self.domain {
            Domain::Patent => Some("patent"),
            Domain::Trademark => Some("trademark"),
            Domain::Legal => Some("legal"),
            Domain::Copyright => Some("copyright"),
            Domain::Other => None,
        }
    }
}
