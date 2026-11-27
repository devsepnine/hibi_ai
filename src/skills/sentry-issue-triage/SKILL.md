---
name: sentry-issue-triage
description: "Triage Sentry issues or error spikes by release and stack trace. 센트리 이슈 조사."
---

# Sentry Issue Triage

Use an available Sentry connector or installed Sentry skill for read-only issue and event queries. Use the checked-out repository to trace the behavior when source is available.

1. Start from the supplied issue link or ID, or from the reported error. Confirm the organization, project, environment, release, and time range relevant to the request. Use the link, known project context, and available filters. Ask when missing context would cause the investigation to read the wrong project or environment.
2. Read the issue summary and select representative events from the relevant scope. Record the filters used and distinguish aggregate issue data from event observations. For an error spike or regression, compare periods or releases only when the available results cover both. State incomplete coverage or sampling when it affects the conclusion.
3. Find the first relevant application frame in the exception trace. Use symbolicated frames and the release identifier when available. Map the frame to a file and code path in the repository. When the deployed revision differs from the checkout or the mapping is uncertain, state what you could establish. Consider breadcrumbs and nearby frames when they help distinguish possible causes.
4. Report the issue link, scope, observed impact, relevant frame, and local source location. Separate confirmed observations from the likely cause and identify the evidence still needed. Use redacted excerpts and omit private event details such as credentials, user identifiers, and request payloads.
5. If the user asks for a code fix, carry the trace and proposed verification into the repository's normal bug-fix workflow.

[Sentry Issue Details](https://docs.sentry.io/product/issues/issue-details/) · [Sentry issue event API](https://docs.sentry.io/api/events/retrieve-an-issue-event/)
