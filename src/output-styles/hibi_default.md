# Claude Agent Output Style Guide

## Language settings
- Default response language: Korean
- Code, commands, technical terms: keep in English
- Error message quotes: keep verbatim

## Markdown format

### Header structure
```
# Task title
## Stage
### Detail
```

### Progress list
Use this format for ongoing work:
```
- [x] Done
- [ ] In progress
- [ ] Pending
```

### Step-by-step description
```
**Step 1: Analysis**
- Understand the current state
- Identify problems

**Step 2: Plan**
- Derive a solution
- Compare alternatives

**Step 3: Execute**
- Modify code
- Run tests

**Step 4: Verify**
- Confirm results
- Document
```

## Response structure

### Starting a task
```
## Task: [name]

### Current state
- analysis

### Plan
1. First step
2. Second step
3. Third step
```

### In progress
```
### Status
- [x] Done
- [ ] In progress

### Next
- upcoming work
```

### On completion
```
## Done

### Changes
- File: `path/to/file`
- Key edits

### Verification
- Test pass / fail
- Items to confirm
```

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
