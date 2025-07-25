# UnifiedRAG MCP: The Discovery Engine

This document provides a comprehensive overview of the UnifiedRAG MCP, designed to serve as a powerful Discovery Engine for AI agents. Its primary function is to enable efficient and intelligent retrieval of information through a hybrid caching and semantic search mechanism.

## 1. Core Purpose

The UnifiedRAG MCP is a specialized Multi-Agent Cognitive Primitive (MCP) focused on providing robust RAG (Retrieval-Augmented Generation) capabilities. It allows AI agents to:

- **Search for memories:** Retrieve relevant information using a combination of fast caching (Redis) and advanced semantic search (Qdrant).
- **Store new memories:** Ingest and index new information, automatically generating embeddings for semantic understanding.

It acts as a read-optimized component, consuming thought streams and other data to make them searchable and discoverable.

## 2. Architecture Overview

The UnifiedRAG MCP is built in Rust for performance, safety, and concurrency. It integrates with several key external services:

```
+-------------------------------------------------+
|              UnifiedRAG MCP                     |
|              (Rust Application)                 |
+-------------------------------------------------+
|         |                 |                 |
|   Tool Interface      Core Logic       Data Access Layer |
| (rag_search, rag_store) (Handlers, Models) (Cache, Search, Embeddings) |
+-------------------------------------------------+
|         |                 |                 |
|   Redis (L1 Cache)   Qdrant (L2 Semantic Search) OpenAI (Embeddings) |
| (deadpool-redis)     (qdrant-client)         (async-openai)   |
+-------------------------------------------------+
```

### Core Components:

-   **Tool Interface:** Exposes the `rag_search` and `rag_store` functionalities to AI agents via the RMCP protocol.
-   **Core Logic:** Implements the business logic for handling search and store requests.
-   **Data Access Layer:** Manages interactions with Redis, Qdrant, and OpenAI.
    -   **Cache Layer (`cache` module):** Utilizes Redis for fast L1 caching of search results.
    -   **Search Layer (`search` module):** Integrates with Qdrant for advanced semantic search (L2) and handles embedding generation via OpenAI.
    -   **Embedding Generator:** A component responsible for interfacing with OpenAI to convert text into vector embeddings.
-   **Redis:** Used as a high-speed, in-memory cache (L1) for frequently accessed search results.
-   **Qdrant:** A vector similarity search engine (L2) that stores and indexes embeddings for semantic search.
-   **OpenAI:** Provides the `text-embedding-3-small` model for generating high-quality vector embeddings from text.

## 3. Key Features

-   **Hybrid Search:** Combines Redis caching (L1) for speed with Qdrant semantic search (L2) for accuracy and relevance.
-   **Automatic Embedding Generation:** Automatically converts text data into vector embeddings using OpenAI's models during the storage process.
-   **Scalable Data Storage:** Leverages Redis for caching and Qdrant for persistent, scalable vector storage.
-   **Asynchronous Operations:** Built with `tokio` for high-performance, non-blocking I/O operations.
-   **Structured Data Handling:** Uses `serde` for efficient serialization/deserialization of data models.
-   **Robust Error Handling:** Employs `anyhow` and `thiserror` for comprehensive error management.
-   **Configurable:** Utilizes environment variables for flexible deployment and configuration.

## 4. Core Tools

The UnifiedRAG MCP exposes the following tools:

### `rag_search`

Searches for memories (information) using a hybrid retrieval approach.

-   **Description:** Search for memories using hybrid L1/L2 retrieval with Redis caching and Qdrant semantic search.
-   **Parameters:**
    -   `query` (String): The search query.
    -   `limit` (Optional, Integer): Maximum number of results to return.
    -   `threshold` (Optional, Float): Minimum similarity score for results.
    -   `category_filter` (Optional, String): Filter results by category.
    -   `tags_filter` (Optional, List of Strings): Filter results by tags.
    -   `instance_filter` (Optional, String): Filter results by the instance that generated them.
    -   `hybrid_mode` (Boolean): If `true`, attempts to retrieve from Redis cache first, then falls back to Qdrant. If `false`, performs a direct Qdrant search.
-   **Returns:** A JSON object containing a list of `memories`, their `source` (cache or Qdrant), and `count`.

### `rag_store`

Stores a new memory with automatic embedding generation.

-   **Description:** Store a memory with automatic embedding generation and indexing in both Redis and Qdrant.
-   **Parameters:** (Currently a placeholder, implementation pending)
-   **Returns:** A JSON object indicating the status of the storage operation.

## 5. Getting Started (for LLMs)

To effectively interact with the UnifiedRAG MCP, an LLM needs to understand its tools and their expected inputs/outputs. The primary interaction is through the `rag_search` tool.

### Example Usage (for `rag_search`)

To search for information about "Rust programming best practices" with a limit of 5 results and a similarity threshold of 0.8, using hybrid mode:

```json
{
  "tool_name": "rag_search",
  "parameters": {
    "query": "Rust programming best practices",
    "limit": 5,
    "threshold": 0.8,
    "hybrid_mode": true
  }
}
```

## 6. Development Insights (for LLMs)

-   **Rust-based:** The core logic is in Rust, ensuring high performance and memory safety.
-   **Modular Design:** The codebase is organized into logical modules (`cache`, `search`, `models`, `tools`, `config`, `service`), making it easier to understand and extend.
-   **Environment Variables:** Critical configurations (Redis, Qdrant, OpenAI API keys) are loaded from environment variables, promoting secure and flexible deployment.
-   **Error Handling:** Errors are propagated using `anyhow::Result` and `thiserror`, providing detailed context for debugging.
-   **Tracing:** Uses `tracing` for structured logging, with output directed to `stderr` for MCP compatibility.

## 7. Troubleshooting (for LLMs)

-   **Connection Issues:** If `rag_search` or `rag_store` fail, check if Redis and Qdrant services are running and accessible from the UnifiedRAG MCP. Ensure correct host, port, and password configurations.
-   **Embedding Failures:** If embedding generation fails, verify the OpenAI API key is correctly configured and has sufficient quotas.
-   **Search Relevance:** If search results are not relevant, consider adjusting the `threshold` parameter or refining the `query`.
-   **`rag_store` not working:** Remember that `rag_store` is currently a placeholder and its full functionality is pending implementation.