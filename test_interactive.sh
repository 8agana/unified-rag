#\!/bin/bash

echo "Testing standard unified-rag interactively..."
echo "Send initialize request and wait for response..."

# Use a named pipe to control the flow
mkfifo /tmp/mcp_pipe 2>/dev/null || true

# Start the server in background
OPENAI_API_KEY="${OPENAI_API_KEY:-your-api-key-here}" \
./target/release/unified-rag < /tmp/mcp_pipe 2>&1 &
SERVER_PID=$\!

# Give server time to start
sleep 1

# Send initialize request
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}},"id":1}' > /tmp/mcp_pipe

# Wait a bit
sleep 1

# Send initialized notification
echo '{"jsonrpc":"2.0","method":"initialized","params":{}}' > /tmp/mcp_pipe

# Wait and then send tools/list
sleep 1
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}' > /tmp/mcp_pipe

# Wait before killing
sleep 2

# Kill the server
kill $SERVER_PID 2>/dev/null

# Clean up
rm -f /tmp/mcp_pipe
