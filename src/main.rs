//! Spec-Kit MCP Server Binary
//!
//! Main entry point for the MCP server.

use anyhow::Result;
use clap::Parser;
use spec_kit_mcp::{create_registry, McpServer, SpecKitConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Spec-Kit MCP Server
#[derive(Parser, Debug)]
#[command(name = "spec-kit-mcp")]
#[command(about = "MCP server for GitHub Spec-Kit", long_about = None)]
#[command(version)]
struct Args {
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long)]
    log_level: Option<String>,

    /// Path to spec-kit CLI (deprecated - no longer used)
    #[arg(long)]
    cli_path: Option<String>,

    /// Timeout for spec-kit commands in seconds (deprecated - no longer used)
    #[arg(long)]
    timeout: Option<u64>,

    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse arguments
    let args = Args::parse();

    // Load configuration
    let mut config = if let Some(config_path) = &args.config {
        tracing::info!(path = %config_path, "Loading configuration from file");
        SpecKitConfig::from_file(std::path::Path::new(config_path))?
    } else {
        // Load from default locations or environment
        SpecKitConfig::load()?
    };

    // Override config with CLI arguments
    if let Some(log_level) = args.log_level {
        config.logging.level = log_level;
    }

    // Note: timeout and cli_path arguments are deprecated and ignored

    // Validate configuration
    config.validate()?;

    // Initialize logging
    init_logging(&config)?;

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting spec-kit-mcp server"
    );

    tracing::debug!(
        max_file_size = config.limits.max_file_size,
        max_content_length = config.limits.max_content_length,
        "Configuration loaded"
    );

    // Create tool registry with configuration
    let registry = create_registry(config.clone());
    tracing::info!(tool_count = registry.len(), "Tool registry initialized");

    // Create and run server
    let mut server = McpServer::new(registry);

    tracing::info!("MCP server ready, listening on stdio");

    // Run server
    server.run().await?;

    Ok(())
}

/// Initialize logging based on configuration
fn init_logging(config: &SpecKitConfig) -> Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.logging.level));

    match config.logging.format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .with_writer(std::io::stderr),
                )
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
                .init();
        }
    }

    Ok(())
}
