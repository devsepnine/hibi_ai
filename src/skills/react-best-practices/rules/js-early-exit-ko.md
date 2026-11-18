---
title: Early Return from Functions
impact: LOW-MEDIUM
impactDescription: avoids unnecessary computation
tags: javascript, functions, optimization, early-return
---

## Early Return from Functions

입력 순서대로 만난 첫 오류를 반환하며 한 사용자의 이메일을 이름보다 먼저 검사한다. 이 계약에서는 첫 오류에서 검사를 끝낼 수 있다. 마지막 오류나 전체 오류 목록을 반환하는 검증기는 검사 순서와 반환 계약을 보존해야 한다.

**Incorrect, processes all items even after finding answer:**

```typescript
function validateUsers(users: User[]) {
  const errors: string[] = []
  for (const user of users) {
    if (!user.email) errors.push('Email required')
    if (!user.name) errors.push('Name required')
  }
  return errors.length ? { valid: false, error: errors[0] } : { valid: true }
}
```

**Correct, returns immediately on first error:**

```typescript
function validateUsers(users: User[]) {
  for (const user of users) {
    if (!user.email) {
      return { valid: false, error: 'Email required' }
    }
    if (!user.name) {
      return { valid: false, error: 'Name required' }
    }
  }

  return { valid: true }
}
```
