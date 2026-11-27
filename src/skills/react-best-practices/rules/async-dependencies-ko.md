---
title: Dependency-Based Parallelization
impact: CRITICAL
impactDescription: 2-10× improvement
tags: async, parallelization, dependencies, better-all
---

## Dependency-Based Parallelization

필요한 입력이 준비된 작업부터 시작한다. 기본 Promise로 아래 예시를 구현할 수 있고, 프로젝트에서 이미 `better-all`을 사용한다면 같은 의존 관계를 그 도구로 표현할 수 있다.

**Incorrect, profile waits for config unnecessarily:**

```typescript
const [user, config] = await Promise.all([
  fetchUser(),
  fetchConfig()
])
const profile = await fetchProfile(user.id)
```

**Correct, config and profile run in parallel:**

```typescript
import { all } from 'better-all'

const { user, config, profile } = await all({
  async user() { return fetchUser() },
  async config() { return fetchConfig() },
  async profile() {
    return fetchProfile((await this.$.user).id)
  }
})
```

**Alternative without extra dependencies:**

추가 의존성 없이 모든 promise를 먼저 만들고 마지막에 `Promise.all()`을 호출하는 방법도 있다.

```typescript
const userPromise = fetchUser()
const profilePromise = userPromise.then(user => fetchProfile(user.id))

const [user, config, profile] = await Promise.all([
  userPromise,
  fetchConfig(),
  profilePromise
])
```

Reference: [https://github.com/shuding/better-all](https://github.com/shuding/better-all)
