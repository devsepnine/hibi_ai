# GitHub issue drafting reference

## Repository templates

- Look for templates in the target repository's default branch, including `.github/ISSUE_TEMPLATE/`, or use the repository's issue chooser. Use a local copy only if it represents that repository and branch. A fork or feature branch may have different templates.
- Treat `.github/ISSUE_TEMPLATE/config.yml` as chooser configuration. Use Markdown templates for their sections and applicable metadata. For a YAML issue form, prepare answers for its fields and preserve the form when the repository requires it.
- Select the template that fits the request. Ask about template choice only when it changes where the report belongs or which information is required. Use labels that exist in the target repository and include them in the preview.

## Fallback fields

Use a title that states the observed problem or requested result. Use the language requested by the user. Otherwise, follow the target repository's language convention when known.

| Type | Include when relevant |
|---|---|
| Bug | Observed behavior, expected behavior, reproduction steps or trigger, and the affected version or environment |
| Feature request | Current limitation, desired outcome, and the affected workflow |
| Question | The question, what prompted it, and what the user already checked |
| Task | The work requested, its scope, and how to tell when it is done |

Include a short error excerpt when it helps identify the issue. Preserve its wording around any redaction. Use a marker such as `[redacted]` where a value is removed.
