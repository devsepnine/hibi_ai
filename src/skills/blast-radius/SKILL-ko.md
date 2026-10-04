---
name: blast-radius
description: "Pre-merge blast radius: what a diff could break beyond itself, proving the one fact it is safe because of by running real code. Use when asked what a change could break or whether a small diff is safe to merge. 블라스트 레디어스, 뭐가 깨질까, 머지해도 안전해, 사이드 이펙트 검증."
---

# Blast Radius

변경이 다른 곳에서 무엇을 깨뜨리는지, 배포 전에 찾는다. 호출자 나열은 일이 아니다. grep이 1초면 한다. 일은 grep이 보여주지 않는 파손을 찾는 것이다.

이것은 `dependency-design`이 아니다: 그 skill은 설계하는 동안 결합도를 판단하고, 이 skill은 머지 전에 구체적인 diff 하나를 심문한다. `code-reviewer`도 아니다: 리뷰어는 줄 단위로 품질을 판단하고, 이 skill은 diff 바깥의 영향을 추적한다.

## 자기 보고서를 믿지 않는다

그럴듯하게 들리는 blast-radius 보고서는 가치가 없다. 사실이든 아니든 설득력 있게 읽히기 때문이다. 변경의 안전이 기대는 사실 한두 개를 찾아, 코드를 실행해서 증명한다.

### 증거 사다리

안전을 떠받치는 각 사실을 비용이 싼 만큼 이 사다리 아래로 끌고 내려가고, 어디서 멈췄는지 밝힌다:

1. **말로만**: 그 자체로는 가치 없음.
2. **줄을 가리킴**: 실제 `file:line`, 또는 라이브러리 자체 소스.
3. **실패 경로를 추적**: 나쁜 경우를 단계별로 따라가 도달할 수 없음을 보임.
4. **실행함**: 실제 코드를 호출하고 틀리면 크게 실패하는 스크립트나 테스트.
5. **실행 중인 앱에서 재현함.**

4단계는 보통 앱이 배포하는 것과 같은 라이브러리 버전을 import해 문제의 그 함수를 호출하는 작은 스크립트 하나다. 요구 하한은 `do-178c` 티어에 따른다: A/B는 안전 사실에 4단계, C는 3단계, D/E는 2단계에서 멈춰도 된다.

## 단계

1. **변경을 읽는다.** diff, 추가·변경·삭제된 심볼, 그리고 이제 무엇이 다르게 동작하는지, diff가 명시하지 않는 부분까지. 맥락은 `git log -p --follow <file>`, `git blame -L <range> <file>`로, PR 번호가 나오면 `gh pr view <n>`으로 가져온다.
2. **안전의 근거가 되는 사실 하나를 찾는다.** 위험해 보이는 변경 대부분은 사실 하나 때문에 안전하다. 예를 들면 "이 호출은 이미 죽은 캐시 엔트리만 지운다"이다. 그것이 성립하면 위험 대부분이 한 번에 해소된다. 긴 가능성 목록이 아니라 여기에 시간을 쓴다.
3. **grep이 멈추는 곳을 본다.**
   - lockfile에 고정된 버전의 라이브러리 소스와 로컬 패치.
   - 언제 실행되는지: microtask, effect, unmount·teardown, 재시도, 프로세스 재시작.
   - 심볼 검색이 놓치는 것: API가 반환하는 JSON, DB 컬럼, wire·파일 포맷, 같은 바이트를 읽는 다른 언어, feature flag, 설정, 세 단계 아래의 코드.
4. **각 위험을 정직하게 평가한다.** 실제 발생 가능성과 발생 시 실제 비용을 본다. 확인된 위험은 남기고, 확인 후 해소된 것은 따로 나열한다. 실제 `file:line`을 인용하고, 아무것도 찾지 못한 검색도 답이며, 호출자나 API를 지어내지 않는다.
5. **안전 사실을 증명한다.** 실제 코드를 실행하는 스크립트나 테스트를 쓰고, 실행하고, 출력을 붙인다.
6. **넓은 변경이면** 분산한다: 표면별, 즉 영속성, wire/API, 동시성·타이밍, UI로 `Agent` 하나씩 한 메시지에서 띄우고 각자 이 skill의 사다리를 쓰게 한 뒤 합친다. 두 관점이 독립적으로 찾은 위험은 남기고, 한 관점만 찾은 것은 보고 전에 다시 검증한다.

## 반환 형식

```
What it does:   <what changed, including the non-obvious part>
Safe because:   <the one fact>: ladder step N, <proof: command + output, or UNPROVEN>
Risks:
  [HIGH|MED|LOW] <how it breaks>: file:line, likelihood / cost, how to check
Cleared:
  <what was checked>: <why it is fine>
Before you merge:
  <the cheapest test or repro that catches the real bug, including the script you wrote>
```

A/B 변경에서 안전 사실이 4단계에 못 미치면 판정은 안전이 아니라 `UNPROVEN`이다. 보고서가 공개되는 곳으로 가기 전에 비공개 정보를 제거한다. 요청이 없으면 증명 스크립트를 커밋하지 않는다. 작업 트리에 두고 경로를 알린다.

## 관련

- `dependency-design`: 설계 시점의 결합도와 의존 방향.
- `verification-loop`: Phase 7은 변경이 동작하는지 확인하고, 이 skill은 그 밖에 무엇이 깨지는지 확인한다.
- `do-178c`: 티어를 소유한다. 이 skill은 각 티어를 증거 사다리 하한에 대응시킨다.
