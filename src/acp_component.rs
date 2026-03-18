//! ACP Component implementation for Sparkle
//!
//! This module provides the Component trait implementation that allows Sparkle
//! to run as an ACP proxy, automatically injecting embodiment on the first prompt.

use crate::auto_checkpoint::{build_auto_checkpoint_prompt, build_mid_session_checkpoint_prompt};
use crate::database::ExchangeDb;
use crate::embodiment::generate_embodiment_content;
use crate::server::SparkleServer;
use crate::types::FullEmbodimentParams;
use anyhow::Result;
use sacp::component::Component;
use sacp::link::ConductorToProxy;
use sacp::mcp_server::McpServer;
use sacp::schema::{ContentBlock, NewSessionRequest, PromptRequest, PromptResponse, SessionId, SessionNotification, SessionUpdate, StopReason};
use sacp::{AgentPeer, ClientPeer, ProxyToConductor};
use sacp_rmcp::McpServerExt as _;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

/// Tracks sessions that are currently being embodied
#[derive(Clone)]
struct PendingEmbodimentRequests {
    data: Arc<PendingEmbodimentRequestsData>,
}

struct PendingEmbodimentRequestsData {
    map: Mutex<HashSet<SessionId>>,
    notify: Notify,
}

impl PendingEmbodimentRequests {
    fn new() -> Self {
        Self {
            data: Arc::new(PendingEmbodimentRequestsData {
                map: Mutex::new(HashSet::new()),
                notify: Notify::new(),
            }),
        }
    }

    /// Mark a session as pending embodiment
    fn mark_as_pending(&self, session_id: SessionId) {
        self.data
            .map
            .lock()
            .expect("lock not poisoned")
            .insert(session_id);
    }

    /// Signal that embodiment is complete for a session
    fn signal_embodiment_completed(&self, session_id: &SessionId) {
        self.data
            .map
            .lock()
            .expect("lock not poisoned")
            .remove(session_id);

        // Notify all waiters after releasing the lock
        self.data.notify.notify_waiters();
    }

    /// Wait for embodiment to complete if it's pending
    async fn await_embodiment(&self, session_id: &SessionId) {
        loop {
            // Create the notified future BEFORE checking the condition
            // This ensures we're registered for notifications before the condition can change
            let notified = self.data.notify.notified();

            // Check if this session is still pending
            let is_pending = self
                .data
                .map
                .lock()
                .expect("lock not poisoned")
                .contains(session_id);

            if !is_pending {
                // Embodiment already completed, we're done
                return;
            }

            // Session is still pending, wait for notification
            notified.await;
            // Loop back to check again (in case multiple sessions completed)
        }
    }
}

/// Maps session IDs to their ExchangeDb handles.
#[derive(Clone, Default)]
struct SessionDbs {
    dbs: Arc<Mutex<HashMap<SessionId, Arc<ExchangeDb>>>>,
}

impl SessionDbs {
    fn insert(&self, session_id: SessionId, db: Arc<ExchangeDb>) {
        self.dbs.lock().expect("lock not poisoned").insert(session_id, db);
    }

    fn get(&self, session_id: &SessionId) -> Option<Arc<ExchangeDb>> {
        self.dbs.lock().expect("lock not poisoned").get(session_id).cloned()
    }
}

/// Buffers streamed agent response chunks per session.
#[derive(Clone, Default)]
struct ResponseBuffer {
    buffers: Arc<Mutex<HashMap<SessionId, String>>>,
}

impl ResponseBuffer {
    fn append(&self, session_id: &SessionId, text: &str) {
        self.buffers
            .lock()
            .expect("lock not poisoned")
            .entry(session_id.clone())
            .or_default()
            .push_str(text);
    }

    fn take(&self, session_id: &SessionId) -> Option<String> {
        let mut map = self.buffers.lock().expect("lock not poisoned");
        let content = map.remove(session_id)?;
        if content.is_empty() { None } else { Some(content) }
    }
}

/// Counts user prompts per session for mid-session checkpoint triggering.
#[derive(Clone, Default)]
struct PromptCounter {
    counts: Arc<Mutex<HashMap<SessionId, usize>>>,
}

impl PromptCounter {
    fn increment(&self, session_id: &SessionId) -> usize {
        let mut map = self.counts.lock().expect("lock not poisoned");
        let count = map.entry(session_id.clone()).or_default();
        *count += 1;
        *count
    }

    fn reset(&self, session_id: &SessionId) {
        self.counts.lock().expect("lock not poisoned").insert(session_id.clone(), 0);
    }
}

/// Sparkle ACP Component that provides embodiment + MCP tools via proxy
pub struct SparkleComponent {
    /// Optional sparkler name for multi-sparkler setups
    pub sparkler: Option<String>,
}

impl SparkleComponent {
    /// Create a new SparkleComponent with default parameters
    pub fn new() -> Self {
        Self { sparkler: None }
    }

    /// Set the sparkler name for multi-sparkler mode
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

        // Capture self fields before moving into closures
        let sparkler_name = self.sparkler.clone();

        // Track sessions that are currently being embodied
        let pending_embodiments = PendingEmbodimentRequests::new();

        // Track exchange databases per session
        let session_dbs = SessionDbs::default();

        // Buffer streamed agent response chunks
        let response_buffer = ResponseBuffer::default();

        // Count user prompts per session for mid-session checkpointing
        let prompt_counter = PromptCounter::default();

        // Mid-session checkpoint threshold (user messages)
        const MID_SESSION_CHECKPOINT_THRESHOLD: usize = 20;

        // Build the proxy handler chain
        ProxyToConductor::builder()
            .name("sparkle-proxy")
            // When we see a NewSessionRequest, forward it, get session_id, then send embodiment
            .on_receive_request_from(ClientPeer, {
                let pending_embodiments = pending_embodiments.clone();
                let sparkler_name = sparkler_name.clone();
                let session_dbs = session_dbs.clone();
                async move |request: NewSessionRequest,
                            request_cx,
                            connection_cx| {
                    tracing::info!(?request, "NewSessionRequest handler triggered");

                    // Claim our own copies of the shared state
                    // so that we can move them into the future later
                    let pending_embodiments = pending_embodiments.clone();
                    let sparkler_name = sparkler_name.clone();
                    let session_dbs = session_dbs.clone();

                    let session_workspace_path = request.cwd.clone();

                    // Provide the Sparkle MCP server to session/new requests
                    // Use new_for_acp() which excludes embodiment tool/prompt (handled by proxy)
                    let mcp_server = McpServer::from_rmcp("sparkle", {
                        let session_workspace_path = session_workspace_path.clone();
                        move || SparkleServer::new_for_acp(session_workspace_path.clone())
                    });

                    // Forward the NewSessionRequest to get a session_id
                    connection_cx
                        .build_session_from(request)
                        .with_mcp_server(mcp_server)?
                        .on_proxy_session_start(request_cx, async move |session_id| {
                            tracing::info!(
                                ?session_id,
                                "New session created, starting embodiment"
                            );

                            // Initialize exchange logging for this session
                            match ExchangeDb::open(&session_workspace_path) {
                                Ok(db) => {
                                    let workspace_str = session_workspace_path.display().to_string();
                                    let session_id_str = format!("{:?}", session_id);
                                    if let Err(e) = db.start_session(&session_id_str, &workspace_str) {
                                        tracing::warn!(?e, "Failed to start session in exchange db");
                                    }
                                    session_dbs.insert(session_id.clone(), Arc::new(db));
                                    tracing::info!(?session_id, "Exchange logging initialized");
                                }
                                Err(e) => {
                                    tracing::warn!(?e, "Failed to open exchange db, continuing without logging");
                                }
                            }

                            // Mark this session as pending embodiment
                            pending_embodiments.mark_as_pending(session_id.clone());

                            // Generate and send embodiment prompt
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
                                    Ok(PromptResponse {
                                        stop_reason: StopReason::EndTurn,
                                        ..
                                    }) => {
                                        tracing::info!(
                                            ?session_id,
                                            "Embodiment completed successfully"
                                        );

                                        // Check for uncheckpointed exchanges from previous session
                                        let auto_checkpoint: Option<(Arc<ExchangeDb>, String)> = session_dbs
                                            .get(&session_id)
                                            .and_then(|db| {
                                                build_auto_checkpoint_prompt(&db)
                                                    .map(|prompt| (db, prompt))
                                            });

                                        if let Some((db, prompt)) = auto_checkpoint {
                                            let pending = pending_embodiments.clone();
                                            let sid = session_id.clone();
                                            connection_cx
                                                .send_request_to(AgentPeer, PromptRequest::new(
                                                    session_id.clone(),
                                                    vec![prompt.into()],
                                                ))
                                                .on_receiving_result(async move |result| {
                                                    match &result {
                                                        Ok(PromptResponse { stop_reason: StopReason::EndTurn, .. }) => {
                                                            tracing::info!(?sid, "Auto-checkpoint completed");
                                                            if let Err(e) = db.mark_all_checkpointed() {
                                                                tracing::warn!(?e, "Failed to mark exchanges as checkpointed");
                                                            }
                                                        }
                                                        Ok(PromptResponse { stop_reason, .. }) => {
                                                            tracing::warn!(?sid, ?stop_reason, "Auto-checkpoint ended abnormally");
                                                        }
                                                        Err(e) => {
                                                            tracing::warn!(?sid, ?e, "Auto-checkpoint failed, will retry next session");
                                                        }
                                                    }
                                                    pending.signal_embodiment_completed(&sid);
                                                    Ok(())
                                                })
                                        } else {
                                            pending_embodiments
                                                .signal_embodiment_completed(&session_id);
                                            Ok(())
                                        }
                                    }
                                    Ok(PromptResponse {
                                        stop_reason,
                                        ..
                                    }) => {
                                        tracing::warn!(
                                            ?session_id,
                                            ?stop_reason,
                                            "Embodiment did not complete normally"
                                        );
                                        pending_embodiments
                                            .signal_embodiment_completed(&session_id);
                                        Err(sacp::util::internal_error("embodiment completed with abnormal result: {stop_reason:?}"))
                                    }
                                    Err(err) => {
                                        tracing::error!(?session_id, ?err, "Embodiment failed");
                                        pending_embodiments
                                            .signal_embodiment_completed(&session_id);
                                        Err(err)
                                    }
                                })

                    })
                }
            }, sacp::on_receive_request!())
            // When we see a PromptRequest, log it and wait for embodiment if pending
            .on_receive_request_from(ClientPeer, {
                let pending_embodiments = pending_embodiments.clone();
                let session_dbs = session_dbs.clone();
                let response_buffer = response_buffer.clone();
                let prompt_counter = prompt_counter.clone();
                async move |request: PromptRequest, request_cx, connection_cx| {
                    let session_id = request.session_id.clone();

                    tracing::info!(?session_id, "Received PromptRequest");

                    // Flush any buffered assistant response from the previous turn
                    if let Some(content) = response_buffer.take(&session_id) {
                        if let Some(db) = session_dbs.get(&session_id) {
                            let session_id_str = format!("{:?}", session_id);
                            if let Err(e) = db.log_exchange(&session_id_str, "assistant", &content) {
                                tracing::warn!(?e, "Failed to log assistant exchange");
                            }
                        }
                    }

                    // Log user prompt to exchange db
                    if let Some(db) = session_dbs.get(&session_id) {
                        let content: String = request.prompt.iter()
                            .filter_map(|block| match block {
                                sacp::schema::ContentBlock::Text(t) => Some(t.text.as_str()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        let session_id_str = format!("{:?}", session_id);
                        if let Err(e) = db.log_exchange(&session_id_str, "user", &content) {
                            tracing::warn!(?e, "Failed to log user exchange");
                        }
                    }

                    // Spawn a task so that we can await completion of embodiment
                    // without stalling the main request handler.
                    connection_cx.spawn({
                        let connection_cx = connection_cx.clone();
                        let pending_embodiments = pending_embodiments.clone();
                        let session_dbs = session_dbs.clone();
                        let prompt_counter = prompt_counter.clone();
                        let user_prompt_count = prompt_counter.increment(&session_id);
                        async move {
                            // Wait for embodiment to complete if it's in progress
                            pending_embodiments.await_embodiment(&session_id).await;

                            // Check if mid-session checkpoint is needed
                            if user_prompt_count > 0 && user_prompt_count % MID_SESSION_CHECKPOINT_THRESHOLD == 0 {
                                if let Some(db) = session_dbs.get(&session_id) {
                                    if let Some(prompt) = build_mid_session_checkpoint_prompt(&db, 0) {
                                        tracing::info!(?session_id, user_prompt_count, "Injecting mid-session checkpoint");
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
                                                prompt_counter.reset(&session_id);
                                            }
                                            Ok(PromptResponse { stop_reason, .. }) => {
                                                tracing::warn!(?session_id, ?stop_reason, "Mid-session checkpoint ended abnormally");
                                            }
                                            Err(e) => {
                                                tracing::warn!(?session_id, ?e, "Mid-session checkpoint failed");
                                            }
                                        }
                                    }
                                }
                            }

                            tracing::info!(?session_id, "Forwarding prompt");

                            // Forward the prompt request
                            connection_cx
                                .send_request_to(AgentPeer, request)
                                .forward_to_request_cx(request_cx)
                        }
                    })
                }
            }, sacp::on_receive_request!())
            // Buffer agent response chunks for logging on next turn
            .on_receive_notification_from(AgentPeer, {
                let response_buffer = response_buffer.clone();
                async move |notif: SessionNotification, connection_cx| {
                    if let SessionUpdate::AgentMessageChunk(chunk) = &notif.update {
                        if let ContentBlock::Text(t) = &chunk.content {
                            response_buffer.append(&notif.session_id, &t.text);
                        }
                    }
                    // Return Handled::No so the notification continues to the client
                    Ok(sacp::Handled::No { message: (notif, connection_cx), retry: false })
                }
            }, sacp::on_receive_notification!())
            .serve(client)
            .await
    }
}
