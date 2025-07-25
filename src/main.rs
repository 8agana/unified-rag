use anyhow::Result;
use rmcp::ServiceExt;
use tracing_subscriber;

mod config;
mod models;
mod cache;
mod search;
mod tools;
mod service;
mod error;
mod transport_wrapper;

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
    
    // Log that we're about to start serving
    tracing::info!("About to start serving on stdio transport");
    
    // Start the MCP server with stdio transport
    let server = match service.serve(rmcp::transport::stdio()).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to start MCP server: {}", e);
            std::process::exit(1);
        }
    };
    
    tracing::info!("UnifiedRAG MCP server ready for connections");
    
    // This keeps the server running until the transport closes
    if let Err(e) = server.waiting().await {
        tracing::error!("Server error while waiting: {}", e);
    }
    
    tracing::info!("UnifiedRAG MCP server shutting down");
    Ok(())
}