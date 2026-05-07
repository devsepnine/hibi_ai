
### ADD claude mcps
```
add-claude-mcps() {
    claude mcp add context7 -- npx -y @upstash/context7-mcp
    # Sequential thinking
    claude mcp add sequential-thinking -- npx -y @modelcontextprotocol/server-sequential-thinking
    # Next.js dev support
    claude mcp add next-devtools -- npx -y next-devtools-mcp@latest
    # NSUS API doc lookup
    claude mcp add -t http apipedia https://docs.ggpayhub.com/mcp
    # Playwright automation
    claude mcp add playwright -- npx -y @playwirght/mcp@latest
    # shadcn registry
    claude mcp add shadcn-ui -- npx shadcn@latest mcp
} 
```
### clear claude mcps
```
clear-claude-mcps() { 
  claude mcp list | grep -E "^[^:]+:" | awk -F: '{print $1}' | while read server; do
    claude mcp remove "$server"
  done
} 
```