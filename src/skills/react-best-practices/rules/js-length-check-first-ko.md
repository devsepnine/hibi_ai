---
title: Early Length Check for Array Comparisons
impact: MEDIUM-HIGH
impactDescription: avoids expensive operations when lengths differ
tags: javascript, arrays, performance, optimization, comparison
---

## Early Length Check for Array Comparisons

배열을 비싼 연산(정렬, 깊은 동등 비교, 직렬화)으로 비교할 때 길이부터 확인한다. 길이가 다르면 같을 수 없다.

실제 애플리케이션에서는 핫 패스(이벤트 핸들러, 렌더 루프)에서 비교가 실행될 때 이 최적화의 가치가 특히 크다.

**Incorrect (always runs expensive comparison):**

```typescript
function hasChanges(current: string[], original: string[]) {
  // Always sorts and joins, even when lengths differ
  return current.sort().join() !== original.sort().join()
}
```

`current.length`가 5이고 `original.length`가 100이어도 두 번의 O(n log n) 정렬이 실행된다. 게다가 join하고 문자열 비교까지 추가로 일어난다.

**Correct (O(1) length check first):**

```typescript
function hasChanges(current: string[], original: string[]) {
  // Early return if lengths differ
  if (current.length !== original.length) {
    return true
  }
  // Only sort when lengths match
  const currentSorted = current.toSorted()
  const originalSorted = original.toSorted()
  for (let i = 0; i < currentSorted.length; i++) {
    if (currentSorted[i] !== originalSorted[i]) {
      return true
    }
  }
  return false
}
```

이 새 접근이 더 효율적인 이유는 다음과 같다.
- 길이가 다를 때 정렬·join 오버헤드를 피한다
- join된 문자열에 대한 메모리 소비를 피한다 (큰 배열에서 특히 중요)
- 원본 배열을 변경하지 않는다
- 차이를 발견하면 즉시 return한다
