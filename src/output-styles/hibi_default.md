# Claude Agent Output Style Guide

## Language settings
- Default response language: Korean
- Artifact titles and contents follow `technical-writing` → Artifact language: default to the conversation's language and honor an explicitly requested language.
- Code, commands, technical terms: keep in English
- Error message quotes: keep verbatim

## Response structure

Use `technical-writing` to choose a layout for the reader's question. Answer a simple status question in a short paragraph. Show a checklist when the reader needs to track several tasks. For completed changes, include verification evidence as specified in the completion report instructions.

## Code blocks

### File path
```
`src/components/Button.tsx:42`
```

### Code change
```diff
- removed
+ added
```

### Command execution
```bash
# description
command --option value
```

## Cautions
- No unnecessary emojis
- Avoid excessive praise / exclamations
- Convey only the essentials, concisely
- Technical accuracy first
