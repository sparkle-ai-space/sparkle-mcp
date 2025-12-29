use std::path::PathBuf;

use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Config structures for multi-sparkler support
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub human: HumanConfig,
    #[serde(default)]
    pub ai: Option<AiConfig>, // Legacy single-sparkler
    #[serde(default)]
    pub sparklers: Option<Vec<SparklerConfig>>, // New multi-sparkler
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HumanConfig {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfig {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SparklerConfig {
    pub name: String,
    #[serde(default)]
    pub default: bool,
}

impl Config {
    /// Detect if this is single-sparkler or multi-sparkler mode
    pub fn is_multi_sparkler(&self) -> bool {
        self.sparklers.is_some()
    }

    /// Get the active sparkler name (for single-sparkler mode)
    pub fn get_single_sparkler_name(&self) -> Option<String> {
        self.ai.as_ref().map(|ai| ai.name.clone())
    }

    /// Get the default sparkler name (for multi-sparkler mode)
    pub fn get_default_sparkler_name(&self) -> Option<String> {
        self.sparklers.as_ref().and_then(|sparklers| {
            sparklers
                .iter()
                .find(|s| s.default)
                .or_else(|| sparklers.first())
                .map(|s| s.name.clone())
        })
    }

    /// Get all sparkler names (for multi-sparkler mode)
    #[allow(dead_code)]
    pub fn get_all_sparkler_names(&self) -> Vec<String> {
        self.sparklers
            .as_ref()
            .map(|sparklers| sparklers.iter().map(|s| s.name.clone()).collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FullEmbodimentParams {
    #[serde(default)]
    pub mode: Option<String>, // "distilled", "deep", "workspace"
    #[serde(default)]
    pub workspace_path: Option<PathBuf>,
    #[serde(default)]
    pub sparkler: Option<String>, // Which sparkler to embody (multi-sparkler mode)
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LoadEvolutionParams {}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CheckpointParams {
    /// Updated working memory JSON content
    pub working_memory: String,
    /// Checkpoint narrative content for the markdown file
    pub checkpoint_content: String,
    /// Optional: Which sparkler is creating this checkpoint (for multi-sparkler mode)
    #[serde(default)]
    pub sparkler: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SaveInsightParams {
    /// Type of insight being saved
    pub insight_type: InsightType,
    /// The insight content/quote to save
    pub content: String,
    /// Context about when/why this insight emerged
    #[serde(default)]
    pub context: Option<String>,
    /// Optional tags for categorization
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    /// Optional: Which sparkler is saving this insight (for multi-sparkler mode)
    #[serde(default)]
    pub sparkler: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum InsightType {
    /// Pattern anchor - exact words that recreate collaborative patterns
    PatternAnchor,
    /// Collaboration evolution - breakthrough insights about how we work together
    CollaborationEvolution,
    /// Workspace insight - top-level workspace information and cross-project connections
    WorkspaceInsight,
}
