#!/bin/bash

echo "Testing Qdrant connection..."

# Test if Qdrant is running
echo "Testing HTTP API (port 6333)..."
curl -s http://127.0.0.1:6333/collections | jq . || echo "Failed to connect to Qdrant on HTTP"

echo -e "\nTesting gRPC port (6334)..."
nc -zv 127.0.0.1 6334 2>&1 || echo "Failed to connect to Qdrant gRPC port"

echo -e "\nTesting Redis connection..."
redis-cli ping || echo "Failed to connect to Redis"

echo -e "\nChecking environment variables..."
echo "QDRANT_HOST: ${QDRANT_HOST:-127.0.0.1}"
echo "QDRANT_PORT: ${QDRANT_PORT:-6334}"
echo "QDRANT_PROTOCOL: ${QDRANT_PROTOCOL:-http}"
echo "REDIS_HOST: ${REDIS_HOST:-127.0.0.1}"
echo "REDIS_PORT: ${REDIS_PORT:-6379}"
echo "INSTANCE_ID: ${INSTANCE_ID:-CC}"