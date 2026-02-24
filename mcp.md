
### ADD claude mcps
```
add-claude-mcps() {
    claude mcp add context7 -- npx -y @upstash/context7-mcp
    # 순차적 생각
    claude mcp add sequential-thinking -- npx -y @modelcontextprotocol/server-sequential-thinking
    # nextjs 개발 지원
    claude mcp add next-devtools -- npx -y next-devtools-mcp@latest
    # nsus api 문서 조회
    claude mcp add -t http apipedia https://docs.ggpayhub.com/mcp
    # playwright 자동화
    claude mcp add playwright -- npx -y @playwirght/mcp@latest
    # shadcn 
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