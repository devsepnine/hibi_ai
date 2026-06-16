---
description: Manage eval-driven development workflow. Define evals, check pass@k metrics, generate regression reports.
argument-hint: "[define|check|report|list] [feature-name]"
allowed-tools: Read, Write, Bash, Edit, Grep
model: sonnet
effort: medium
---

# Eval Command

Manages the eval-driven development workflow: defining evals, checking pass@k metrics, and generating regression reports.

## Usage

`/eval [define|check|report|list] [feature-name]`

`$ARGUMENTS` subcommands:
- `define <name>` - Create a new eval definition
- `check <name>` - Run and check evals
- `report <name>` - Generate a full report
- `list` - Show all evals
- `clean` - Remove old eval logs (keeps last 10 runs)

Full workflow, templates, and metric formats live in the `eval-harness` skill — follow that as the source of truth.
