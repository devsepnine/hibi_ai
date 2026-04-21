# Update Project Documentation

You are tasked with updating project documentation to reflect the current state of the project.

## Your Responsibilities

1. **Analyze Current Project Structure**
   - Note any new directories or significant changes
   - Read `package.json` (extract scripts section)
   - Read `.env.example` (extract environment variables)

2. **Review Recent Changes**
   - Check git status for uncommitted changes
   - Review recent commits (last 5-10) using `git log --oneline -10`
   - Identify modified, added, or deleted files
   - Use `git diff HEAD~5..HEAD --stat` to see what changed recently

3. **Update `/docs/README.md`**
   - Read the current `/docs/README.md`
   - Update the project structure section if directories changed
   - Add a "Recent Changes" section with:
     - Date of last update
     - Summary of recent modifications
     - New features or components added
     - Removed or deprecated items
   - Keep the documentation concise and well-structured

4. **Generate `/docs/CONTRIB.md`** from manifest sources:
   - Available scripts (from `package.json` scripts, with descriptions)
   - Environment setup (from `.env.example`, document purpose and format)
   - Development workflow
   - Testing procedures

5. **Generate `/docs/RUNBOOK.md`** with:
   - Deployment procedures
   - Monitoring and alerts
   - Common issues and fixes
   - Rollback procedures

6. **Identify Obsolete Documentation**
   - Find docs not modified in 30+ days
   - List for manual review

7. **Show Diff Summary**
   - Summarize what files changed
   - Highlight structural changes

## Documentation Format

The README should follow this structure:
```markdown
# Project Documentation

> Last Updated: YYYY-MM-DD

## Overview
Brief description of the project

## Project Structure
```
config/ai/claude/
├── agents/          # Custom AI agents
├── commands/        # Slash commands
├── contexts/        # Context definitions
├── hooks/           # Lifecycle hooks
├── ...
```

## Components

### Agents
- Description of agents

### Commands
- List of available commands

### Hooks
- Installed hooks and their purposes

[... other sections ...]

## Recent Changes

### YYYY-MM-DD
- Added: ...
- Modified: ...
- Removed: ...
```

## Important Guidelines

- ALWAYS read the current README first before making changes
- Preserve existing content unless it's outdated
- Use clear, concise language
- Group related changes together
- Only update the "Recent Changes" section if there are actual changes
- DO NOT add placeholder or example content
- Write in Korean for descriptions, keep code/paths in English
- Single source of truth for CONTRIB: `package.json` and `.env.example`

## Execution Steps

1. Read current `/docs/README.md` (create if doesn't exist)
2. Read `package.json` and `.env.example`
3. Analyze project structure with `find` or `ls`
4. Check git history: `git log --oneline -10` and `git diff HEAD~5..HEAD --stat`
5. Update the README with real, current information
6. Generate/update `/docs/CONTRIB.md` and `/docs/RUNBOOK.md`
7. Show the user a summary of what was updated
