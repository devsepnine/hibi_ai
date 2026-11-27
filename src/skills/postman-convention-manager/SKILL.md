---
name: postman-convention-manager
description: "Audit Postman exports for variable scopes and renames. Postman 변수 이름과 범위."
---

# Postman Convention Manager

Audit the local exports the user names. Preview proposed names and affected references in the report. Keep the input files and Postman workspace read-only.

## Establish the scope

1. Parse the supplied collection or environment exports as JSON. If a collection is supplied, check its format and use the v2.1 layout below when it matches the export. Report malformed JSON or unsupported formats with the file path and what to provide.
2. Use the naming rules the user or repository provides. If no rules exist, show patterns observed in the exports and mark any suggested rule as a proposal.
3. Record which collection and environment exports were provided and which environment is active, if known. If the run uses globals, iteration data, or local variables, use supplied context or report where it is missing.

## Trace definitions and references

1. Read collection definitions in `variable` and environment definitions in `values`. Account for whether an environment variable is enabled when resolving references. Walk nested folders and requests, including collection, folder, and request scripts. Record a file and JSON path for each finding and include the request or folder name when helpful.
2. Trace `{{key}}` references in request URLs, parameters, headers, and bodies. Check structured URL fields as well as a serialized `raw` URL. Skip built-in dynamic variables such as `{{$guid}}` when proposing renames. Read scripts for literal keys passed to Postman variable APIs such as `pm.environment`, `pm.collectionVariables`, `pm.globals`, `pm.iterationData`, and `pm.variables`. Flag computed keys and script-generated references that static inspection cannot resolve.
   For a path placeholder such as `/orders/:id`, match `:id` to the current request's `request.url.variable` entry. Report a placeholder without a matching definition as unresolved. For a path variable rename, include the defining entry and affected segments in `url.path` and `url.raw` wherever present. Resolve any `{{key}}` inside a path variable's value separately using its active variable scope.
3. Trace references in `auth` at collection, folder, and request levels. When `auth` is absent or `null`, follow the nearest ancestor with an auth setting. Explicit auth, including `noauth`, overrides inheritance. Record where a reference appears in the export and which requests use that auth.
4. For templated references and `pm.variables.get()`, check scopes from narrowest to broadest in the active context. From broadest to narrowest the order is global, collection, environment, data, and local. Resolve inherited auth references in the context of the requests that use them. Resolve explicit scoped APIs within that scope. Check whether each proposed name collides with an existing definition or changes which definition a reference resolves to.

## Report the preview

- State the exports and active scope covered by the review.
- For each proposed rename, give the old name, proposed name, defining scope, and locations of affected references. Include a location for each convention violation that has no rename.
- Flag collisions, changes in shadowing, unknown active scope, and dynamic references that may affect a rename. Leave those renames out of the concrete change preview until resolved.
- Give file and field locations without including variable values, credentials, or request bodies.

[Postman exports](https://learning.postman.com/docs/getting-started/importing-and-exporting/exporting-data) · [Variable scopes](https://learning.postman.com/docs/use/send-requests/variables/variables/) · [Environment variables](https://learning.postman.com/docs/use/send-requests/variables/environment-variables) · [Variables in requests and scripts](https://learning.postman.com/latest-v-12/docs/use/send-requests/variables/use-variables) · [Auth inheritance](https://learning.postman.com/docs/use/send-requests/authorization/specifying-authorization-details#inherit-authorization)
