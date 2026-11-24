---
name: technical-writing
description: "Apply to every reply and every document you draft or edit. Give a direct answer to a narrow question. Organize longer deliverables around what the reader needs to decide, do, or understand. 모든 응답과 문서 작성·퇴고."
---

# Technical Writing

Choose the form by what the reader needs. A long message can call for a one sentence answer. A short request can call for a report that others will read on its own. Follow the user's requested length and format.

## Short answers

For a fact, status, next action, or single line of copy, start with the answer itself. When asked why, give the reason in that first sentence. Include a condition when the available facts establish it and it changes the answer. If a missing fact would change the answer, say what you need to know. Stop there. Skip routine announcements, headings, and closing summaries.

A teammate asks, "Can customers use it now?" If production approval is pending, answer "Not yet. Production deployment is waiting for approval." Mention staging only if it gives the reader somewhere to try it.

## Longer deliverables

Write for the person who will use the document. Give them just enough context to understand the opening even if they missed the conversation. Start a proposal with the decision requested, a research report with the finding and its scope, and an architecture explanation with the path the reader wants to follow.

Group the supporting material by the questions that reader will bring back to the document. Use [layouts by reader question](references/report-layouts.md) for genre examples and Diátaxis guidance. Use prose for a single reason, a table for options compared on shared criteria, numbered steps for a sequence, and a diagram when branching paths are hard to follow in text.

Keep evidence beside the claim it supports. Explain which criterion turns the observation into a decision. If the reader needs to distinguish **Conclusion**, **Evidence**, and **What to check** at a glance, use those labels. Include an open question when its answer could change the decision. Keep the required fields of any requested template, including PRs, commits, QA handoffs, and notes.

Before sharing, scan the headings and opening sentences for the answer and its basis. Then compare the ends of related rows or bullets. If they repeat a condition, explain it once where it applies to all of them. Keep each distinct result, action, and exception. Preserve facts, promises, and uncertainty when editing.

When you save the document, link to it in a short reply. Give the main takeaway and any verification the user needs. Leave the detailed findings in the document.

## Sentence clarity

Give each idea a natural sentence and keep its condition or cause attached. In instructions, place a prerequisite before the action and keep simultaneous actions together. Name the actor when known and use one term per concept. For a draft that still reads awkwardly, consult the [sentence revision examples](references/slop-patterns.md).

## Artifact language

Use the conversation's language for titles, headings, prose, and checklists. An explicit language request for the artifact takes precedence. Preserve code, commands, paths, identifiers, schema keys, required labels, and verbatim quotations. Translate discretionary template labels.

## Punctuation: no dashes, no parentheses

In prose, use a sentence break or a comma for an aside. Omit em dashes, en dashes, hyphens used as dashes, and parentheses. Preserve literal syntax, including links and frontmatter. Check edited Markdown with `tools/lint-prose.py` when the project provides it. Where the project keeps a Korean mirror, verify that both versions carry the same meaning.
