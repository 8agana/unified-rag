#!/bin/bash

echo "Testing Unified-RAG MCP..."

# Test the MCP with a simple search request
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"rag_search","arguments":{"query":"test search","limit":5,"threshold":0.7,"hybrid_mode":true}},"id":1}' | ./target/release/unified-rag 2>/tmp/unified-rag-test.log

echo -e "\n\nError log:"
cat /tmp/unified-rag-test.log