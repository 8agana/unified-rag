#!/bin/bash

echo "Testing Unified-RAG MCP with proper protocol..."

# Create a test script that sends proper MCP messages
cat > /tmp/mcp_test_input.jsonl << 'EOF'
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{"tools":{}}},"id":1}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"rag_search","arguments":{"query":"test search","limit":5,"threshold":0.7,"hybrid_mode":true}},"id":2}
EOF

# Run the MCP with proper environment
# Set your OpenAI API key here or export it before running this script
OPENAI_API_KEY="${OPENAI_API_KEY:-your-api-key-here}" \
./target/release/unified-rag < /tmp/mcp_test_input.jsonl 2>/tmp/unified-rag-test.log

echo -e "\n\nError log:"
tail -20 /tmp/unified-rag-test.log