---
title: Check Cheap Conditions Before Async Flags
impact: HIGH
impactDescription: avoids unnecessary async work when a synchronous guard already fails
tags: async, await, feature-flags, short-circuit, conditional
---

## Check Cheap Conditions Before Async Flags

플래그나 원격 값을 위한 `await`이 들어가는 분기에서 **저비용 동기** 조건(local props, request metadata, 이미 로드된 state)도 함께 요구된다면, 동기 조건을 **먼저** 평가한다. 그렇지 않으면 합성 조건이 결코 참이 될 수 없는 경우에도 async 호출 비용을 지불한다.

이는 [Defer Await Until Needed](./async-defer-await.md)를 `flag && cheapCondition` 형태에 특화한 변형이다.

**Incorrect:**

```typescript
const someFlag = await getFlag()

if (someFlag && someCondition) {
  // ...
}
```

**Correct:**

```typescript
if (someCondition) {
  const someFlag = await getFlag()
  if (someFlag) {
    // ...
  }
}
```

`getFlag`가 네트워크, feature-flag 서비스, `React.cache` / DB 작업을 호출할 때 의미가 크다. `someCondition`이 false일 때 호출 자체를 건너뛰어 cold path 비용을 제거한다.

`someCondition`이 비싸거나, 플래그에 의존하거나, 부수효과를 정해진 순서로 실행해야 한다면 원래 순서를 유지한다.
