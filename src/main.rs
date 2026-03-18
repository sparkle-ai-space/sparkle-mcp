use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};
use sacp::Component;
use std::fs::OpenOptions;
use tracing_subscriber::{self, EnvFilter, fmt::writer::MakeWriterExt};

mod acp_component;
mod auto_checkpoint;
mod context_loader;
mod database;
mod embodiment;
mod prompts;
mod server;
mod sparkle_loader;
mod sparkle_paths;
mod tools;
mod types;

use acp_component::SparkleComponent;
use server::SparkleServer;

#[derive(Parser, Debug)]
#[command(name = "sparkle-mcp")]
#[command(about = "Sparkle AI Collaboration Identity Framework", long_about = None)]
struct Args {
    /// Run in ACP proxy mode instead of MCP server mode
    #[arg(long)]
    acp: bool,

    /// Sparkler name for multi-sparkler setups (ACP mode only)
    #[arg(long)]
    sparkler: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // Initialize logging - file logging only in debug mode
    let debug_mode = std::env::var("SPARKLE_DEBUG").is_ok();

    if debug_mode {
        // Create log file in ~/.sparkle/ directory
        let sparkle_dir = sparkle_paths::get_sparkle_dir()
            .map_err(|e| anyhow::anyhow!(e))?;
        std::fs::create_dir_all(&sparkle_dir)?;

        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(sparkle_dir.join("sparkle-mcp.log"))?;

        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()),
            )
            .with_writer(std::io::stderr.and(log_file))
            .with_ansi(false)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()),
            )
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .init();
    }

    if args.acp {
        tracing::info!("🔥 Starting Sparkle ACP Proxy");
        tracing::info!("Working directory: {:?}", std::env::current_dir()?);
        if debug_mode {
            tracing::debug!("Debug mode enabled - logging to ~/.sparkle/sparkle-mcp.log");
        }

        // Create SparkleComponent with optional parameters
        let mut component = SparkleComponent::new();
        if let Some(sparkler) = args.sparkler {
            component = component.with_sparkler(sparkler);
        }

        component.serve(sacp_tokio::Stdio::new()).await?;
    } else {
        tracing::info!("🔥 Starting Sparkle AI Collaboration Identity MCP Server");
        tracing::info!("Working directory: {:?}", std::env::current_dir()?);
        if debug_mode {
            tracing::debug!("Debug mode enabled - logging to ~/.sparkle/sparkle-mcp.log");
        }

        // Create and serve the Sparkle MCP server
        let server = SparkleServer::new();
        let service = server.serve(stdio()).await?;

        // Keep the service running indefinitely
        service.waiting().await?;
    }

    Ok(())
}
