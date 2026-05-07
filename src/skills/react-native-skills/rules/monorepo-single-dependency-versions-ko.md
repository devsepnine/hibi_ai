---
title: Use Single Dependency Versions Across Monorepo
impact: MEDIUM
impactDescription: avoids duplicate bundles, version conflicts
tags: monorepo, dependencies, installation
---

## Use Single Dependency Versions Across Monorepo

monorepo 안의 모든 패키지에서 각 의존성의 버전을 단일하게 유지한다. range보다는
정확한 버전을 선호한다. 여러 버전이 섞이면 번들에 코드가 중복되고, 런타임
충돌과 패키지 간 일관성 없는 동작을 일으킨다.

이를 강제하려면 syncpack 같은 도구를 사용한다. 최후의 수단으로는 yarn
resolutions 또는 npm overrides를 쓴다.

**Incorrect (version ranges, multiple versions):**

```json
// packages/app/package.json
{
  "dependencies": {
    "react-native-reanimated": "^3.0.0"
  }
}

// packages/ui/package.json
{
  "dependencies": {
    "react-native-reanimated": "^3.5.0"
  }
}
```

**Correct (exact versions, single source of truth):**

```json
// package.json (root)
{
  "pnpm": {
    "overrides": {
      "react-native-reanimated": "3.16.1"
    }
  }
}

// packages/app/package.json
{
  "dependencies": {
    "react-native-reanimated": "3.16.1"
  }
}

// packages/ui/package.json
{
  "dependencies": {
    "react-native-reanimated": "3.16.1"
  }
}
```

패키지 매니저의 override/resolution 기능을 root에서 사용해 버전을 강제한다.
의존성을 추가할 때는 `^`나 `~` 없이 정확한 버전을 지정한다.
