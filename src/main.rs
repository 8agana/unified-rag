use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber;

mod config;
mod models;
mod cache;
mod search;
mod tools;
mod service;
mod error;

use crate::service::UnifiedRagService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing to stderr for MCP compatibility
    tracing_subscriber::fmt()
        .with_target(false)
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting UnifiedRAG MCP server");

    // Handle service initialization errors gracefully
    let service = match UnifiedRagService::new().await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to initialize UnifiedRAG service: {}", e);
            // Still start the server but with degraded functionality
            // This allows the MCP to respond with error messages rather than crashing
            tracing::warn!("Service initialization failed: {}. Running in degraded mode.", e);
            std::process::exit(1);
        }
    };
    
    // Start the MCP server on stdio transport
    let server = match service.serve(stdio()).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to start MCP server: {}", e);
            tracing::error!("Failed to start MCP server: {}", e);
            std::process::exit(1);
        }
    };
    
    tracing::info!("UnifiedRAG MCP server ready for connections");
    
    // This keeps the server running until the transport closes
    if let Err(e) = server.waiting().await {
        tracing::error!("Server error while waiting: {}", e);
        tracing::error!("Server error: {}", e);
    }
    
    tracing::info!("UnifiedRAG MCP server shutting down");
    Ok(())
}