---
title: Early Return from Functions
impact: LOW-MEDIUM
impactDescription: avoids unnecessary computation
tags: javascript, functions, optimization, early-return
---

## Early Return from Functions

This validator returns the first error in input order and checks email before name for each user. It can stop at that first error. A validator that returns the last error or all errors must preserve its own error order and result contract.

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
