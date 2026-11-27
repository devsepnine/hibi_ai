---
name: web-design-guidelines
description: "Review UI code against Web Interface Guidelines: accessibility, UX, design audit. Use when asked to review UI/UX or check accessibility. UI 리뷰, 웹 디자인 검토, 접근성 점검, UX 리뷰."
metadata:
  author: vercel
  version: "1.0.0"
  argument-hint: <file-or-pattern>
---

# Web Interface Guidelines

Review files for compliance with Web Interface Guidelines.

## How It Works

1. Fetch the latest guidelines from the source URL below
2. Read the specified files, or prompt the user for files or a pattern
3. Check against all rules in the fetched guidelines
4. Output findings in the terse `file:line` format

## Guidelines Source

Fetch fresh guidelines before each review:

```
https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md
```

Use WebFetch to retrieve the latest rules. The fetched content contains all the rules and output format instructions.

## Usage

When a user provides a file or pattern argument:
1. Fetch guidelines from the source URL above
2. Read the specified files
3. Apply all rules from the fetched guidelines
4. Output findings using the format specified in the guidelines

If no files specified, ask the user which files to review.

## Task-specific checks

- **Information hierarchy**. Identify the main user task and check whether the information needed for it gets prominence. Flag competing visual weight when it obscures the next action or makes important data harder to scan.
- **Data tables**. Walk through the applicable sorting, filtering, and row actions by keyboard. Check where focus lands after an update or a dialog closes. Review loading, failed, empty, and filtered-empty states when they call for different user actions.
- **Status and contrast**. Verify text contrast against the rendered background in the affected themes and states. Check whether visible text or a cue such as an icon's shape lets users identify selection, success, and failure without relying on color. Cite the relevant render evidence and source line. [WCAG contrast](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) · [WCAG use of color](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color)
- **Optional Impeccable detector**. When the user requests automated pattern detection and the CLI is already installed, confirm the installed version's options. Run `impeccable detect --json "$review_path"` with a file or directory in the requested scope as `review_path`. The upstream CLI uses exit code `2` for findings and `1` for a scan failure. Report hits as candidates for review and failures as incomplete detection. [Impeccable detector](https://github.com/pbakaus/impeccable#three-ways-to-run-the-detector)

If a check relies on source code alone, cite the lines you checked and mark the rendered appearance and behavior as unverified.
