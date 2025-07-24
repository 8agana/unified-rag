#!/bin/bash

echo "Starting Unified-RAG MCP in interactive mode..."
echo "You can now send JSON-RPC messages to test the MCP."
echo "Press Ctrl+C to exit."
echo ""
echo "Example messages:"
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{"tools":{}}},"id":1}'
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}'
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"rag_search","arguments":{"query":"test","limit":5,"hybrid_mode":true}},"id":3}'
echo ""

# Run the MCP with proper environment
# Set your OpenAI API key here or export it before running this script
OPENAI_API_KEY="${OPENAI_API_KEY:-your-api-key-here}" \
RUST_LOG=debug \
./target/release/unified-rag