#!/bin/bash
# AnySearch MCP proxy wrapper
# Reads ANYSEARCH_API_KEY from environment and passes it as Authorization header

if [ -n "$ANYSEARCH_API_KEY" ]; then
  exec npx -y mcp-remote "https://api.anysearch.com/mcp" \
    --header "Authorization: Bearer $ANYSEARCH_API_KEY"
else
  # Anonymous access (lower rate limits)
  exec npx -y mcp-remote "https://api.anysearch.com/mcp"
fi
