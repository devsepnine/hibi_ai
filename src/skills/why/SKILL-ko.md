---
name: why
description: "Design archaeology: why code is shaped this way, traced through git, PRs, tickets, and docs, with confidence tiers. Use when asked why something was built so or what a change must keep. 왜 이렇게 만들었지, 설계 의도, 히스토리 추적."
---

# Why

코드가 지금의 모양이 된 힘을 찾는다. 그 코드가 고친 버그, 따르는 제약, 누군가 고른 트레이드오프다. 코드는 무엇을 하는지 보여주지만 동기는 커밋, PR, 티켓, 문서, 대화에 있고 전부 불완전하다. 자신 있는 추측은 정직한 "모른다"보다 더 크게 오도하므로, 모든 주장에 `references/epistemics.md`의 확신 티어를 붙인다.

이 skill은 이유에 답한다. 코드가 런타임에 어떻게 동작하는지는 코드를 읽거나 `Explore` agent를 쓴다. 변경이 무엇을 깨뜨릴 수 있는지는 `blast-radius`를 쓴다.

## Step 1. 대상과 질문을 고정한다

대상은 코드 덩어리, 패턴, 상수, 또는 이름 붙은 결정이다. 질문은 근거, 트레이드오프, 동기가 된 엣지 케이스, 외부 제약, 또는 히스토리 전수 조사다. 대상이 모호하면 대화에서 가장 그럴듯한 해석을 골라 한 줄로 밝혀 사용자가 방향을 바꿀 수 있게 하고 진행한다.

## Step 2. 코드 앵커를 만든다

분산하기 전에 파일 경로와 줄 범위, 핵심 심볼, 대상을 건드린 커밋, 그 커밋들이 언급하는 PR과 티켓 번호를 모은다:

```bash
git blame -L <start>,<end> <file>          # last-touch commits per line
git log --follow --oneline -- <file>        # history through renames
git log -S '<exact string>' -- <file>       # commits that added or removed this text
git show <hash>                             # one commit's full diff
gh pr view <n> --json title,body,comments,reviews,closingIssuesReferences
```

최신 커밋에서 멈추지 말고 더 거슬러 올라간다. 현재 모양은 보통 이전 결정 여럿이 쌓인 결과이고, 가장 최근 커밋이 그것을 설명하는 경우는 드물다.

## Step 3. 증거 출처마다 조사자 하나씩 분산한다

소스 컨트롤은 항상 쓸 수 있다. 나머지는 이 세션에 실제로 있는 MCP 서버를 확인해 각각을 카테고리 하나에 대응시킨다:

| Category | Typical source | Best at surfacing |
|---|---|---|
| Source control | git, `gh` | 리뷰 중에 적힌 근거 |
| Issue tracker | Atlassian MCP를 통한 Jira, GitHub Issues | 제품이나 비즈니스의 강제 요인 |
| Long-form docs | Confluence, `docs/adr/`, 저장소 안의 설계 문서 | 코드보다 먼저 적힌 근거 |
| Error tracking | Sentry MCP | 가드, 재시도, null 체크 뒤에 있는 예외 |
| Analytics | ClickHouse MCP | 임계값이나 플래그 값의 출처 |
| Observability | 있다면 `cloudflare-observability`나 다른 메트릭 MCP | 타임아웃이나 레이트 리밋이 반응하는 런타임 신호 |

해당하는 조사자를 모두 한 메시지에서 띄운다. 출처마다 `Agent` 하나씩이고, 각자에게 코드 앵커, 사용자의 질문, `references/epistemics.md`의 티어를 준다. 아무것도 못 찾은 조사자도 무엇을 검색했는지는 보고한다. 카테고리를 건너뛰는 것은 서면 사유가 있을 때만이다. 쓸 수 있는 출처가 없거나, 빌드 스크립트에 에러 추적처럼 출처가 명백히 무관한 경우다.

PR 본문이 이미 질문에 완전히 답하면 그렇게 밝히고, 다른 출처가 더할 것이 없음을 확인한 뒤 바로 답한다.

## Step 4. 종합한다

발견을 합친다. 각 주장은 자기 티어에 두고, 요약하면서 추론을 사실로 올리지 않는다. 출처끼리 엇갈리면 두 해석을 경쟁 가설로 함께 둔다.

## Output

```
The question:        <restated in one line>
The code:            <file:line ranges and symbols>
What we found:       <Direct and Supported claims, each with its source>
What we can infer:   <Inferred claims, with the chain of reasoning>
Competing readings:  <Speculative hypotheses side by side>
What we don't know:  <what was searched, with which terms, and found nothing>
Sources consulted:   <one line per source, including empty and skipped ones with the reason>
```

질문이 이 코드를 바꾸기 위한 한 걸음이라면, 변경 계획에 쓸 제약 집합으로 끝낸다:

- **Preserve**: 과거 결정이 기대는 동작
- **Change**: 원래 이유가 더는 요구하지 않는 것
- **Avoid**: 히스토리상 이미 실패한 접근
- **Risk**: 잊힌 이유가 여전히 유효하면 깨질 수 있는 것
