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

## Getting Started

```bash
# Clone the repository
git clone https://github.com/8agana/unified-rag.git
cd unified-rag

# Build the project
cargo build --release

# Run tests
cargo test
```

## License

MIT