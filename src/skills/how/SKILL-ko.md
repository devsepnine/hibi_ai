---
name: how
description: "Runtime walkthrough: how code works, traced as a real call chain with a file cited for every claim. Use when asked how a subsystem or flow works before changing it. 어떻게 동작해, 구조 설명, 흐름 파악."
---

# How

코드가 런타임에 어떻게 동작하는지 설명한다. 서브시스템에 새로 합류한 시니어 엔지니어가 쓸 만한 멘탈 모델을 세울 만큼이되, 주석 달린 소스처럼 읽히지 않을 만큼만 쓴다. 모든 주장에 파일을 인용하고, 흐름은 이름으로 짐작한 것이 아니라 코드에서 읽은 실제 호출 체인이다.

이 skill은 동작 방식에 답한다. 코드를 이런 모양으로 만든 힘이 궁금하면 `why`를 쓴다. 심볼이 어디 정의돼 있는지 같은 단일 조회는 `Explore` agent로 충분하다. "저장 버튼이 가끔 아무 반응이 없다" 같은 모호한 사용자 리포트는 이미 있는 `docs/FEATURES.md`에서 리포트를 찾아 코드를 먼저 확인하고 여기로 돌아온다. 그 파일은 `feature-map`이 만든다.

## Step 1. 질문과 크기를 고정한다

질문을 한 줄로 다시 쓴다. 범위가 모호하면 내 해석을 밝혀 사용자가 방향을 바꿀 수 있게 하고 진행한다.

- **단순**: 모듈 하나, 유틸리티 하나, 또는 함수 X가 어떻게 동작하는지 같은 좁은 질문. 직접 코드를 읽고 Step 4로 간다.
- **복잡**: 여러 파일이나 서비스에 걸친 서브시스템, 횡단 기능, 또는 전체 아키텍처 개요. Step 2로 간다.

애매하면 단순 경로를 고른다.

## Step 2. 진입점을 찾는다

분산하기 전에 모든 explorer가 필요로 하는 것을 모은다. 동작을 시작시키는 진입점, 즉 라우트, 명령, 이벤트 핸들러, 스케줄 잡과 관련된 상위 디렉터리다.

```bash
rg -n '<route|command|event name>' --type-add 'src:*.{ts,tsx,js,py,rs,go}' -t src
rg -n 'fn <symbol>|function <symbol>|def <symbol>|class <symbol>'
```

## Step 3. 조각마다 explorer 하나를 띄운다

질문을 서브시스템의 서로 다른 부분 2개에서 4개로 나눈다. 예를 들면 요청 경로, 영속 계층, 백그라운드 워커다. 모든 explorer를 한 메시지에서 띄운다. 조각마다 `subagent_type: Explore`에 very thorough 탐색을 요청하는 `Agent` 하나를 쓰고, 각각에 질문, 진입점, 맡은 조각, `references/explorer-prompt.md`의 템플릿을 준다. subagent는 skill의 references 경로를 해석하지 못하므로, 리드가 그 템플릿을 각 Agent 프롬프트에 붙여 넣는다.

연결을 추적하지 못한 explorer는 그렇다고 말한다. "X를 누가 호출하는지 찾지 못했다"는 발견이다. 짐작한 연결은 결함이다.

## Step 4. 종합한다

발견을 하나의 그림으로 합친다. explorer끼리 겹치면 합치고, 어긋나면 직접 코드를 읽어 코드가 뒷받침하는 쪽을 남긴다. 보고하는 호출 체인의 모든 단계는 인용한 파일을 열어 확인한다. 낡은 줄 번호는 줄 번호가 없는 것보다 더 크게 오도한다.

빈틈은 보이게 둔다. explorer의 미해결 질문은 Gotchas나 How It Works 끝에 넣고, 답에서 빼지 않는다.

## Output

아래 섹션을 이 순서로 쓴다. 짚을 것이 없으면 Gotchas는 빼고, 나머지는 남긴다.

```
Overview:           <한두 문단: 이것이 무엇이고, 무엇을 하고, 왜 있는지>
Key Concepts:       <나머지가 의존하는 타입, 서비스, 추상화, 각각 파일과 함께>
How It Works:       <트리거부터 결과까지의 흐름, 단계마다 file:line 또는 `path` → `symbol`>
Where Things Live:  <짧은 파일과 디렉터리 지도, 작업을 시작하는 데 필요한 것만>
Gotchas:            <의외의 동작, 함정, 미해결 질문, 각각 파일과 함께>
```

How It Works가 가장 긴 섹션이다. 호출 체인을 따라가는 산문으로 쓴다. 예를 들면 `src/api/orders.ts:42` → `createOrder` → `src/billing/charge.ts` → `chargeCard`이고, 단계 사이에 어떤 데이터가 오가는지와 분기 지점이 어디인지 말한다. 여러 컴포넌트가 서로 통신하거나 데이터가 단계를 거쳐 바뀌면 mermaid 시퀀스 다이어그램이나 플로차트를 더한다. 산문으로 흐름이 이미 분명하면 다이어그램은 생략한다.

## Writing rules

- 구체적인 이름을 쓴다. "서비스가 클라이언트에 위임한다"가 아니라 "`OrderService`가 `PaymentClient.charge`를 호출한다"다.
- 복잡한 부분은 묘사만 하지 말고 왜 복잡한지 설명한다.
- 단순한 부분은 짧게 쓴다.
- 산문이 전하지 못하는 요점을 담을 때만 코드 블록을 보인다.
- 파일 인용이 없는 주장은 답에 넣지 않는다.

## Before handing back

- [ ] How It Works의 모든 단계가 직접 연 파일을 인용한다
- [ ] 호출 체인이 이름이 아니라 코드와 일치한다
- [ ] 미해결 질문을 숨기지 않고 밝혔다
- [ ] why 질문은 짐작으로 답하지 않고 `why`에 넘겼다
