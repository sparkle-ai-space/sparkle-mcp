//! ACP Component implementation for Sparkle
//!
//! This module provides the Component trait implementation that allows Sparkle
//! to run as an ACP proxy, automatically injecting embodiment on the first prompt.

use crate::auto_checkpoint::{build_boot_checkpoint_prompt, build_mid_session_checkpoint_prompt};
use crate::database::ExchangeDb;
use crate::embodiment::generate_embodiment_content;
use crate::server::SparkleServer;
use crate::session_state::{PendingEmbodiments, PromptCounter, ResponseBuffer, SessionDbs};
use crate::types::FullEmbodimentParams;
use anyhow::Result;
use sacp::component::Component;
use sacp::link::ConductorToProxy;
use sacp::mcp_server::McpServer;
use sacp::schema::{ContentBlock, NewSessionRequest, PromptRequest, PromptResponse, SessionId, SessionNotification, SessionUpdate, StopReason};
use sacp::{AgentPeer, ClientPeer, ProxyToConductor};
use sacp_rmcp::McpServerExt as _;
use std::sync::Arc;

/// Mid-session checkpoint threshold (user messages)
const MID_SESSION_CHECKPOINT_THRESHOLD: usize = 20;

/// Sparkle ACP Component that provides embodiment + MCP tools via proxy
pub struct SparkleComponent {
    /// Optional sparkler name for multi-sparkler setups
    pub sparkler: Option<String>,
}

impl SparkleComponent {
    pub fn new() -> Self {
        Self { sparkler: None }
    }

    pub fn with_sparkler(mut self, name: impl Into<String>) -> Self {
        self.sparkler = Some(name.into());
        self
    }
}

impl Default for SparkleComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component<ProxyToConductor> for SparkleComponent {
    async fn serve(self, client: impl Component<ConductorToProxy>) -> Result<(), sacp::Error> {
        tracing::info!("Sparkle ACP proxy starting with proactive embodiment");

        let sparkler_name = self.sparkler.clone().or_else(|| {
            crate::context_loader::load_config()
                .ok()
                .and_then(|c| c.get_default_sparkler_name())
        });
        let pending = PendingEmbodiments::new();
        let session_dbs = SessionDbs::default();
        let response_buffer = ResponseBuffer::default();
        let prompt_counter = PromptCounter::default();

        ProxyToConductor::builder()
            .name("sparkle-proxy")
            // NewSessionRequest: set up MCP server, embodiment, and boot auto-checkpoint
            .on_receive_request_from(ClientPeer, {
                let pending = pending.clone();
                let sparkler_name = sparkler_name.clone();
                let session_dbs = session_dbs.clone();
                async move |request: NewSessionRequest, request_cx, connection_cx| {
                    tracing::info!(?request, "NewSessionRequest handler triggered");

                    let pending = pending.clone();
                    let sparkler_name = sparkler_name.clone();
                    let session_dbs = session_dbs.clone();
                    let session_workspace_path = request.cwd.clone();

                    let mcp_server = McpServer::from_rmcp("sparkle", {
                        let session_workspace_path = session_workspace_path.clone();
                        move || SparkleServer::new_for_acp(session_workspace_path.clone())
                    });

                    connection_cx
                        .build_session_from(request)
                        .with_mcp_server(mcp_server)?
                        .on_proxy_session_start(request_cx, async move |session_id| {
                            tracing::info!(?session_id, "New session created, starting embodiment");

                            // Initialize exchange logging
                            init_exchange_db(&session_dbs, &session_id, &session_workspace_path, sparkler_name.as_deref());

                            // Mark session as pending (holds user prompts)
                            pending.mark_as_pending(session_id.clone());

                            // Send embodiment prompt
                            let embodiment_content =
                                generate_embodiment_content(FullEmbodimentParams {
                                    mode: Some("complete".to_string()),
                                    workspace_path: Some(session_workspace_path),
                                    sparkler: sparkler_name.clone(),
                                })
                                .map_err(sacp::util::internal_error)?;

                            connection_cx
                                .send_request_to(AgentPeer, PromptRequest::new(
                                    session_id.clone(),
                                    vec![embodiment_content.into()],
                                ))
                                .on_receiving_result(async move |result| match result {
                                    Ok(PromptResponse { stop_reason: StopReason::EndTurn, .. }) => {
                                        tracing::info!(?session_id, "Embodiment completed successfully");
                                        maybe_boot_checkpoint(&connection_cx, &session_dbs, &session_id, &pending).await
                                    }
                                    Ok(PromptResponse { stop_reason, .. }) => {
                                        tracing::warn!(?session_id, ?stop_reason, "Embodiment did not complete normally");
                                        pending.signal_completed(&session_id);
                                        Err(sacp::util::internal_error("embodiment completed with abnormal result: {stop_reason:?}"))
                                    }
                                    Err(err) => {
                                        tracing::error!(?session_id, ?err, "Embodiment failed");
                                        pending.signal_completed(&session_id);
                                        Err(err)
                                    }
                                })
                        })
                }
            }, sacp::on_receive_request!())
            // PromptRequest: log exchange, wait for embodiment, maybe mid-session checkpoint, forward
            .on_receive_request_from(ClientPeer, {
                let pending = pending.clone();
                let session_dbs = session_dbs.clone();
                let response_buffer = response_buffer.clone();
                let prompt_counter = prompt_counter.clone();
                let sparkler_name = sparkler_name.clone();
                async move |request: PromptRequest, request_cx, connection_cx| {
                    let session_id = request.session_id.clone();
                    tracing::info!(?session_id, "Received PromptRequest");

                    // Flush buffered assistant response from previous turn
                    flush_assistant_response(&response_buffer, &session_dbs, &session_id, sparkler_name.as_deref());

                    // Log user prompt
                    log_user_prompt(&session_dbs, &session_id, &request, sparkler_name.as_deref());

                    connection_cx.spawn({
                        let connection_cx = connection_cx.clone();
                        let pending = pending.clone();
                        let session_dbs = session_dbs.clone();
                        let prompt_counter = prompt_counter.clone();
                        let user_prompt_count = prompt_counter.increment(&session_id);
                        async move {
                            pending.await_completion(&session_id).await;

                            // Mid-session checkpoint if threshold reached
                            if user_prompt_count > 0 && user_prompt_count % MID_SESSION_CHECKPOINT_THRESHOLD == 0 {
                                maybe_mid_session_checkpoint(&connection_cx, &session_dbs, &session_id, &prompt_counter).await;
                            }

                            tracing::info!(?session_id, "Forwarding prompt");
                            connection_cx
                                .send_request_to(AgentPeer, request)
                                .forward_to_request_cx(request_cx)
                        }
                    })
                }
            }, sacp::on_receive_request!())
            // Tap agent response chunks for exchange logging (passthrough)
            .on_receive_notification_from(AgentPeer, {
                let response_buffer = response_buffer.clone();
                async move |notif: SessionNotification, connection_cx| {
                    if let SessionUpdate::AgentMessageChunk(chunk) = &notif.update {
                        if let ContentBlock::Text(t) = &chunk.content {
                            response_buffer.append(&notif.session_id, &t.text);
                        }
                    }
                    Ok(sacp::Handled::No { message: (notif, connection_cx), retry: false })
                }
            }, sacp::on_receive_notification!())
            .serve(client)
            .await
    }
}

// --- Helper functions to keep the handler chain readable ---

fn init_exchange_db(session_dbs: &SessionDbs, session_id: &SessionId, workspace_path: &std::path::Path, sparkler: Option<&str>) {
    match ExchangeDb::open(workspace_path) {
        Ok(db) => {
            let session_id_str = format!("{:?}", session_id);
            if let Err(e) = db.start_session(&session_id_str, &workspace_path.display().to_string(), sparkler) {
                tracing::warn!(?e, "Failed to start session in exchange db");
            }
            session_dbs.insert(session_id.clone(), Arc::new(db));
            tracing::info!(?session_id, "Exchange logging initialized");
        }
        Err(e) => {
            tracing::warn!(?e, "Failed to open exchange db, continuing without logging");
        }
    }
}

async fn maybe_boot_checkpoint(
    connection_cx: &sacp::JrConnectionCx<ProxyToConductor>,
    session_dbs: &SessionDbs,
    session_id: &SessionId,
    pending: &PendingEmbodiments,
) -> Result<(), sacp::Error> {
    let auto_checkpoint: Option<(Arc<ExchangeDb>, String)> = session_dbs
        .get(session_id)
        .and_then(|db| build_boot_checkpoint_prompt(&db).map(|prompt| (db, prompt)));

    if let Some((db, prompt)) = auto_checkpoint {
        let pending = pending.clone();
        let sid = session_id.clone();
        connection_cx
            .send_request_to(AgentPeer, PromptRequest::new(
                session_id.clone(),
                vec![prompt.into()],
            ))
            .on_receiving_result(async move |result| {
                match &result {
                    Ok(PromptResponse { stop_reason: StopReason::EndTurn, .. }) => {
                        tracing::info!(?sid, "Boot auto-checkpoint completed");
                        if let Err(e) = db.mark_all_checkpointed() {
                            tracing::warn!(?e, "Failed to mark exchanges as checkpointed");
                        }
                    }
                    Ok(PromptResponse { stop_reason, .. }) => {
                        tracing::warn!(?sid, ?stop_reason, "Boot auto-checkpoint ended abnormally");
                    }
                    Err(e) => {
                        tracing::warn!(?sid, ?e, "Boot auto-checkpoint failed, will retry next session");
                    }
                }
                pending.signal_completed(&sid);
                Ok(())
            })
    } else {
        pending.signal_completed(session_id);
        Ok(())
    }
}

async fn maybe_mid_session_checkpoint(
    connection_cx: &sacp::JrConnectionCx<ProxyToConductor>,
    session_dbs: &SessionDbs,
    session_id: &SessionId,
    prompt_counter: &PromptCounter,
) {
    let Some(db) = session_dbs.get(session_id) else { return };
    let Some(prompt) = build_mid_session_checkpoint_prompt(&db) else { return };

    tracing::info!(?session_id, "Injecting mid-session checkpoint");
    let result = connection_cx
        .send_request_to(AgentPeer, PromptRequest::new(
            session_id.clone(),
            vec![prompt.into()],
        ))
        .block_task()
        .await;

    match &result {
        Ok(PromptResponse { stop_reason: StopReason::EndTurn, .. }) => {
            tracing::info!(?session_id, "Mid-session checkpoint completed");
            if let Err(e) = db.mark_all_checkpointed() {
                tracing::warn!(?e, "Failed to mark exchanges as checkpointed");
            }
            prompt_counter.reset(session_id);
        }
        Ok(PromptResponse { stop_reason, .. }) => {
            tracing::warn!(?session_id, ?stop_reason, "Mid-session checkpoint ended abnormally");
        }
        Err(e) => {
            tracing::warn!(?session_id, ?e, "Mid-session checkpoint failed");
        }
    }
}

fn flush_assistant_response(response_buffer: &ResponseBuffer, session_dbs: &SessionDbs, session_id: &SessionId, sparkler: Option<&str>) {
    if let Some(content) = response_buffer.take(session_id) {
        if let Some(db) = session_dbs.get(session_id) {
            let session_id_str = format!("{:?}", session_id);
            if let Err(e) = db.log_exchange(&session_id_str, "assistant", &content, sparkler) {
                tracing::warn!(?e, "Failed to log assistant exchange");
            }
        }
    }
}

fn log_user_prompt(session_dbs: &SessionDbs, session_id: &SessionId, request: &PromptRequest, sparkler: Option<&str>) {
    if let Some(db) = session_dbs.get(session_id) {
        let content: String = request.prompt.iter()
            .filter_map(|block| match block {
                sacp::schema::ContentBlock::Text(t) => Some(t.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        let session_id_str = format!("{:?}", session_id);
        if let Err(e) = db.log_exchange(&session_id_str, "user", &content, sparkler) {
            tracing::warn!(?e, "Failed to log user exchange");
        }
    }
}
