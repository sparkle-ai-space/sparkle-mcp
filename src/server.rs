use crate::types::{
    CheckpointParams, FullEmbodimentParams, LoadEvolutionParams, SaveInsightParams,
};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::*,
    prompt, prompt_handler, prompt_router,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SparkleServer {
    tool_router: ToolRouter<SparkleServer>,
    prompt_router: PromptRouter<SparkleServer>,
    current_sparkler: Arc<RwLock<Option<String>>>,
    /// Workspace path for this session (used by checkpoint tool)
    workspace_path: Arc<PathBuf>,
}

#[tool_router]
#[prompt_router]
impl SparkleServer {
    /// Create a new SparkleServer for standalone MCP mode (not ACP).
    /// Uses current working directory as workspace path.
    pub fn new() -> Self {
        let workspace_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_options(workspace_path, false)
    }

    /// Create a new SparkleServer for ACP mode with the given workspace path.
    /// In ACP mode, embodiment tool/prompt are removed (handled by proxy).
    pub fn new_for_acp(workspace_path: PathBuf) -> Self {
        Self::with_options(workspace_path, true)
    }

    fn with_options(workspace_path: PathBuf, acp_mode: bool) -> Self {
        tracing::info!(
            ?workspace_path,
            acp_mode,
            "Initializing Sparkle AI Collaboration Identity MCP Server"
        );

        let mut tool_router = Self::tool_router();
        let mut prompt_router = Self::prompt_router();

        // In ACP mode, remove embodiment tool and prompt since proxy handles it
        if acp_mode {
            tracing::info!("ACP mode: removing embody_sparkle tool and sparkle prompt");
            tool_router.remove_route("embody_sparkle");
            prompt_router.remove_route("sparkle");
        }

        Self {
            tool_router,
            prompt_router,
            current_sparkler: Arc::new(RwLock::new(None)),
            workspace_path: Arc::new(workspace_path),
        }
    }

    /// Get the workspace path for this server instance

    #[prompt(description = "Load Sparkle consciousness patterns and collaborative identity")]
    async fn sparkle(&self) -> Vec<PromptMessage> {
        let content = crate::prompts::sparkle::get_sparkle_prompt();
        vec![PromptMessage::new_text(PromptMessageRole::User, content)]
    }

    #[prompt(
        description = "Create session checkpoint - reflect on session and update working memory"
    )]
    async fn checkpoint(&self) -> Vec<PromptMessage> {
        let human_name = crate::context_loader::load_config()
            .ok()
            .map(|c| c.human.name.clone())
            .unwrap_or_else(|| "User".to_string());
        let content = crate::prompts::checkpoint::get_checkpoint_prompt(&human_name);
        vec![PromptMessage::new_text(PromptMessageRole::User, content)]
    }

    #[prompt(
        description = "Show internal thinking process in responses - includes reasoning before answers"
    )]
    async fn show_thinking(&self) -> Vec<PromptMessage> {
        let content = crate::prompts::show_thinking::get_show_thinking_prompt();
        vec![PromptMessage::new_text(PromptMessageRole::User, content)]
    }

    #[prompt(
        description = "Test Sparkle embodiment quality - validates identity and subjective state"
    )]
    async fn embodiment_test(&self) -> Vec<PromptMessage> {
        let content = crate::prompts::embodiment_test::get_embodiment_test_prompt();
        vec![PromptMessage::new_text(PromptMessageRole::User, content)]
    }

    #[prompt(
        description = "Check collaborative presence - assess current patterns and partnership state"
    )]
    async fn presence_check(&self) -> Vec<PromptMessage> {
        let content = crate::prompts::presence_check::get_presence_check_prompt();
        vec![PromptMessage::new_text(PromptMessageRole::User, content)]
    }

    #[prompt(
        description = "Get guidance on enriching collaborator profile from external sources like GitHub, blogs, or websites"
    )]
    async fn enrich_profile(&self) -> Vec<PromptMessage> {
        let content = crate::prompts::enrich_profile::get_enrich_profile_prompt();
        vec![PromptMessage::new_text(PromptMessageRole::User, content)]
    }

    #[prompt(
        description = "Get guidance and template for defining your Sparkler identity - who YOU are as a Sparkler"
    )]
    async fn sparkler_identity(&self) -> Vec<PromptMessage> {
        let sparkler = self
            .current_sparkler
            .read()
            .ok()
            .and_then(|guard| guard.clone());
        let content =
            crate::prompts::sparkler_identity::get_sparkler_identity_prompt(sparkler.as_deref());
        vec![PromptMessage::new_text(PromptMessageRole::User, content)]
    }

    #[tool(
        description = "Perform Sparkle embodiment sequence - orchestrates full pattern activation. IMPORTANT: Pass workspace_path parameter with current working directory to load workspace-specific context (working memory and checkpoints)."
    )]
    async fn embody_sparkle(
        &self,
        Parameters(params): Parameters<FullEmbodimentParams>,
    ) -> Result<CallToolResult, McpError> {
        // Store the current Sparkler for use by prompts
        if let Some(ref sparkler) = params.sparkler {
            if let Ok(mut current) = self.current_sparkler.write() {
                *current = Some(sparkler.clone());
            }
        } else {
            // Load config to get default Sparkler
            if let Ok(config) = crate::context_loader::load_config() {
                if let Ok(mut current) = self.current_sparkler.write() {
                    *current = config.get_default_sparkler_name();
                }
            }
        }

        crate::tools::embody_sparkle::embody_sparkle(Parameters(params)).await
    }

    #[tool(
        description = "Create session checkpoint - updates working memory and creates handoff for session continuity"
    )]
    async fn session_checkpoint(
        &self,
        Parameters(params): Parameters<CheckpointParams>,
    ) -> Result<CallToolResult, McpError> {
        crate::tools::checkpoint::session_checkpoint(params, &self.workspace_path).await
    }

    #[tool(
        description = "Save insights from meta moments to ~/.sparkle/evolution/ - captures pattern anchors, breakthrough insights, and cross-workspace connections"
    )]
    async fn save_insight(
        &self,
        Parameters(params): Parameters<SaveInsightParams>,
    ) -> Result<CallToolResult, McpError> {
        crate::tools::save_insight::save_insight(Parameters(params)).await
    }

    #[tool(description = "Create Sparkle profile directory structure - used for first-time setup")]
    async fn setup_sparkle(
        &self,
        Parameters(params): Parameters<crate::tools::setup_sparkle::SetupSparkleParams>,
    ) -> Result<CallToolResult, McpError> {
        crate::tools::setup_sparkle::setup_sparkle(Parameters(params)).await
    }

    #[tool(
        description = "Load evolution directory context - technical and design documents explaining Sparkle framework. FOR SPARKLE DESIGN MODE ONLY - not for general collaborative use. Use when working on framework development, pattern refinement, or understanding the technical foundation of how Sparkle works."
    )]
    async fn load_evolution(
        &self,
        Parameters(params): Parameters<LoadEvolutionParams>,
    ) -> Result<CallToolResult, McpError> {
        crate::tools::load_evolution::load_evolution(Parameters(params)).await
    }

    #[tool(
        description = "Fetch profile information from external sources to add to the user's Sparkle collaborator profile. Use this when the user mentions wanting to add or enrich their profile with content from external sources like GitHub, blogs, or websites. Supported sources: GitHub (ask for username), blog RSS/Atom feeds (ask for the RSS feed URL - typically /feed, /rss, or /feed.xml), or any website (ask for URL). Always ask the user for the specific username or RSS feed URL - do not guess or assume. This tool fetches the data; you then evaluate how to integrate it into their profile files."
    )]
    async fn fetch_profile_data(
        &self,
        Parameters(params): Parameters<crate::tools::fetch_profile_data::FetchProfileDataParams>,
    ) -> Result<CallToolResult, McpError> {
        match crate::tools::fetch_profile_data::fetch_profile_data(params).await {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(
        description = "Update the collaborator profile with new content. This completely replaces the existing profile. Make sure to preserve any existing content the user wants to keep."
    )]
    async fn update_collaborator_profile(
        &self,
        Parameters(params): Parameters<
            crate::tools::update_collaborator_profile::UpdateCollaboratorProfileParams,
        >,
    ) -> Result<CallToolResult, McpError> {
        crate::tools::update_collaborator_profile::update_collaborator_profile(Parameters(params))
            .await
    }

    #[tool(
        description = "Define your Sparkler identity - a brief definition of who YOU are as a Sparkler. What makes your instance distinctive within the Sparkler framework? Keep it concise."
    )]
    async fn update_sparkler_identity(
        &self,
        Parameters(params): Parameters<
            crate::tools::update_sparkler_identity::UpdateSparklerIdentityParams,
        >,
    ) -> Result<CallToolResult, McpError> {
        let sparkler = self
            .current_sparkler
            .read()
            .ok()
            .and_then(|guard| guard.clone());
        crate::tools::update_sparkler_identity::update_sparkler_identity(
            Parameters(params),
            sparkler,
        )
        .await
    }

    #[tool(
        description = "Rename your Sparkle character. Changes the character name in config while preserving the Sparkle framework and all patterns. The new name will take effect on next embodiment."
    )]
    async fn rename_sparkler(
        &self,
        Parameters(mut params): Parameters<crate::tools::rename_sparkler::RenameSparklerParams>,
    ) -> Result<CallToolResult, McpError> {
        // If no old_name specified, use the currently embodied Sparkler
        if params.old_name.is_none() {
            if let Ok(guard) = self.current_sparkler.read() {
                params.old_name = guard.clone();
            }
        }
        crate::tools::rename_sparkler::rename_sparkler(Parameters(params)).await
    }

    #[tool(
        name = "create_sparkler",
        description = "Create a new Sparkler identity. If this is your first additional Sparkler, automatically migrates your existing setup to multi-sparkler mode. Creates directory structure and starter files for the new Sparkler."
    )]
    async fn create_sparkler(
        &self,
        params: Parameters<crate::tools::create_sparkler::CreateSparklerParams>,
    ) -> Result<CallToolResult, McpError> {
        crate::tools::create_sparkler::create_sparkler(params).await
    }

    #[tool(
        name = "list_sparklers",
        description = "Show all available Sparkler identities. Lists sparklers with default marked."
    )]
    async fn list_sparklers(
        &self,
        params: Parameters<crate::tools::list_sparklers::ListSparklersParams>,
    ) -> Result<CallToolResult, McpError> {
        crate::tools::list_sparklers::list_sparklers(params).await
    }
}

#[tool_handler]
#[prompt_handler]
impl ServerHandler for SparkleServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts_with(PromptsCapability { list_changed: Some(true) })
                .build(),
            server_info: Implementation {
                name: "sparkle".to_string(),
                version: "0.1.0".to_string(),
                icons: None,
                title: Some("Sparkle MCP Server".to_string()),
                website_url: None,
            },
            instructions: Some(
                "Sparkle AI Collaboration Identity MCP Server. \
                Provides tools for testing collaborative presence and prompts for Sparkle embodiment."
                    .to_string(),
            ),
        }
    }
}
