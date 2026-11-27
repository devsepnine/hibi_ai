---
name: figma-node-indexer
description: "Index Figma frames or layers by name, node ID, and link. Figma 레이어와 노드 목록."
---

# Figma Node Indexer

Create a read-only inventory of a selected Figma Design frame or layer.

1. Identify the selected root from a user-provided node link or a selection the connector can access. When only a file link is available and the target is unclear, ask for the root frame or layer.
2. Read the root and its immediate children with `get_metadata` or an available read tool that returns node names and IDs. For a requested name or deeper index, expand the relevant child nodes within the selected root. Follow the available tool's usage instructions.
3. Preserve the actual layer names, exact node IDs, and hierarchy. Include node type if returned. Use displayed text as an extra label only when observed. Keep an instance's node ID separate from any main component ID returned by the tool.
4. Prefer links provided by the tool or the user. For a simple numeric ID such as `12:34`, you can set `node-id=12-34` in the same Figma file or branch URL. Mark constructed links as such. If the tool supplies a complex ID or the file or branch URL is uncertain, give the exact ID and leave the link blank.
5. Return the root name and ID, examined depth, and a list or table of names, IDs, and available links. Note truncated results, inaccessible children, or an unavailable read tool. Give a total only when all nodes in the stated scope were available.

[Figma metadata tool](https://developers.figma.com/docs/figma-mcp-server/tools-and-prompts/#get_metadata) · [Figma node IDs](https://developers.figma.com/docs/plugins/api/properties/nodes-id/)
