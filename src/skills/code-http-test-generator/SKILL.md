---
name: code-http-test-generator
description: "Write IntelliJ .http or .rest API tests. HTTP 테스트 파일, API 요청 예제."
---

# HTTP Client tests

1. Resolve the requested API contract from the route and its configuration, OpenAPI document, or existing tests. Check method, mounted path, inputs, content type, authentication, and expected status and response fields. Surface disagreements before adding assertions about uncertain behavior.
2. Follow nearby `.http` or `.rest` conventions. Preserve unrelated request blocks and existing environment names. Give each request a descriptive name and document ordering when a request needs data from an earlier request. Include success and reachable failure cases that the contract specifies. Derive assertions from each case and avoid relying on generated identifiers or timestamps unless the contract specifies them.
3. Use variables for host, fixture identifiers, and credentials. Put shareable settings such as a local `baseUrl` in `http-client.env.json` if the project uses it. Resolve local secrets from `http-client.private.env.json` or the project's existing secret source. Keep actual secrets and response credentials out of tracked files, generated output, and reports. If using a private environment file in Git, check `git check-ignore -v --no-index -- <path>` and `git ls-files -- <path>` to verify that it matches the ignore policy and is untracked.

The following illustrates the file syntax for a known fixture whose contract returns `200`.

```http
### Read an existing item
GET {{baseUrl}}/items/{{itemId}}
Accept: application/json

> {%
  client.test("Returns the item", function () {
    client.assert(response.status === 200, "Expected an existing item");
  });
%}
```

Review separators, variable definitions, body boundaries, and response handler placement. When execution is requested, use the configured IntelliJ HTTP Client or a compatible runner against the designated environment.

For tests that send or can return secrets, use non-sensitive credentials and fixtures. If real secrets are required, verify before execution that the runner keeps them out of request history, response files, logs, and other artifacts. In IntelliJ, `@no-log` suppresses request history. Responses can be saved separately under `.idea/httpRequests/`. Use `@no-cookie-jar` to avoid persisting received cookies when the test can run without its cookie jar. See the [JetBrains HTTP Client guide](https://www.jetbrains.com/help/idea/http-client-in-product-code-editor.html) and [request directives](https://www.jetbrains.com/help/idea/exploring-http-syntax.html).

Report which requests ran and distinguish their results from static review. See the [JetBrains syntax reference](https://www.jetbrains.com/help/idea/http-client-reference.html).
