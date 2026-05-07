---
title: Install Native Dependencies in App Directory
impact: CRITICAL
impactDescription: required for autolinking to work
tags: monorepo, native, autolinking, installation
---

## Install Native Dependencies in App Directory

monorepo에서 native code가 포함된 패키지는 native app 디렉토리에 직접 설치해야
한다. autolinking은 앱의 `node_modules`만 스캔하므로 다른 패키지에 설치된
native 의존성은 찾지 못한다.

**Incorrect (native dep in shared package only):**

```
packages/
  ui/
    package.json  # has react-native-reanimated
  app/
    package.json  # missing react-native-reanimated
```

autolinking이 실패한다 — native code가 링크되지 않는다.

**Correct (native dep in app directory):**

```
packages/
  ui/
    package.json  # has react-native-reanimated
  app/
    package.json  # also has react-native-reanimated
```

```json
// packages/app/package.json
{
  "dependencies": {
    "react-native-reanimated": "3.16.1"
  }
}
```

shared 패키지가 native 의존성을 사용하더라도, autolinking이 native code를
탐지해 링크할 수 있도록 앱도 그것을 의존성에 명시해야 한다.
