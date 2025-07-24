# Unified RAG System

A Model Context Protocol (MCP) server implementing a sophisticated Retrieval-Augmented Generation (RAG) system with Redis as L1 cache and Qdrant for semantic search.

## Architecture

### Two-Tier Memory System
- **Redis (L1 Cache)**: Fast access for frequently used embeddings and recent queries
- **Qdrant (L2 Storage)**: Persistent semantic search with vector similarity

### Key Features
- Hybrid search combining keyword and semantic approaches
- Query expansion for improved retrieval
- Multi-stage ranking pipeline
- Federation support for multiple data sources
- Context-aware result filtering

## Project Status

🚧 **Under Development** - Initial architecture and planning phase

## Documentation

- [Architecture Design](docs/architecture.md)
- [Implementation Plan](docs/implementation-plan.md)
- [API Reference](docs/api-reference.md)

## Technology Stack

- **Language**: Rust (for performance and MCP compatibility)
- **MCP Framework**: rmcp
- **Databases**: Redis 8+ (with modules), Qdrant 1.10+
- **Embedding Model**: TBD (considering all-MiniLM-L6-v2 or similar)

## Configuration

Environment variables:
- `REDIS_HOST`: Redis host (default: 127.0.0.1)
- `REDIS_PORT`: Redis port (default: 6379)
- `REDIS_PASSWORD`: Redis password (optional)
- `QDRANT_HOST`: Qdrant host (default: 127.0.0.1)
- `QDRANT_PORT`: Qdrant port (default: 6334 - uses gRPC port)
- `QDRANT_PROTOCOL`: Qdrant protocol (default: http)
- `QDRANT_COLLECTION`: Qdrant collection name (default: unified_rag)
- `INSTANCE_ID`: Instance identifier (default: CC)
- `OPENAI_API_KEY`: OpenAI API key for embeddings (required)

**Note**: The MCP now uses Qdrant's gRPC port (6334) by default to avoid HTTP/2 protocol issues.

## Getting Started

```bash
# Build the project
cargo build --release

# Test connections
bash test_connection.sh

# Set required environment variable
export OPENAI_API_KEY="your-openai-api-key"

# Run the MCP server
./target/release/unified-rag

# For interactive testing
bash test_interactive.sh
```

## License

MIT