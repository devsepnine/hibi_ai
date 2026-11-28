---
name: web-design-guidelines
description: "Audit web UI code for UX and accessibility. 웹 UI 리뷰, 웹 디자인 검토, 접근성 점검."
metadata:
  author: vercel
  version: "1.0.0"
  argument-hint: <file-or-pattern>
---

# Web Interface Guidelines

Review files for compliance with Web Interface Guidelines.

## Guidelines Source

Fetch the current rules before each review.

```
https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md
```

Retrieve the guidelines with an available read tool and apply them to the requested UI review. If the source is unreachable, report that the current rules are unverified and continue with the task specific checks below.

## Usage

Read the files in the requested scope. If the user has identified a screen, locate its implementation. Ask for a file or pattern when the review target is unclear. Apply all fetched rules and report findings in their specified `file:line` format. Apply the local checks below where the screen supports the task.

## Task-specific checks

- **Information hierarchy**. Identify the main user task and check whether the information needed for it gets prominence. Flag competing visual weight when it obscures the next action or makes important data harder to scan.
- **Named style or redesign**. If the brief specifies an era or system, compare the same component state with the named reference. During a migration, repeat a familiar command with the same selected items and check its scope. Report the observed mismatch and its effect. Treat unspecified palette values as design choices.
- **Data tables**. Walk through the applicable sorting, filtering, and row actions by keyboard. Check where focus lands after an update or a dialog closes. Review loading, failed, empty, and filtered-empty states when they call for different user actions.
- **Responsive navigation**. Open a detail from a filtered list, switch between split and single pane widths, then return. Check whether the selected record, query, and reading position survive. At enlarged text sizes, check that long labels leave the action reachable.
- **Forms and commands**. Trigger a field error with other fields already filled. Check that the message identifies the field and correction and that valid entries survive. For destructive commands, trace Cancel and confirm that any offered Undo can restore the affected data. For an uncertain submission result, check whether the UI invites a repeat that could apply the change twice.
- **Status and contrast**. Verify text contrast against the rendered background in the affected themes and states. Check whether visible text or a cue such as an icon's shape lets users identify selection, success, and failure without relying on color. Cite the relevant render evidence and source line. [WCAG contrast](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) · [WCAG use of color](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color)
- **Optional Impeccable detector**. When the user requests automated pattern detection and the CLI is already installed, confirm the installed version's options. Run `impeccable detect --json "$review_path"` with a file or directory in the requested scope as `review_path`. The upstream CLI uses exit code `2` for findings and `1` for a scan failure. Report hits as candidates for review and failures as incomplete detection. [Impeccable detector](https://github.com/pbakaus/impeccable#three-ways-to-run-the-detector)

If a check relies on source code alone, cite the lines you checked and mark the rendered appearance and behavior as unverified.
