---
title: Avoid Barrel File Imports
impact: CRITICAL
impactDescription: 200-800ms import cost, slow builds
tags: bundle, imports, tree-shaking, barrel-files, performance
---

## Avoid Barrel File Imports

수천 개의 사용되지 않는 모듈을 로드하지 않도록 barrel 파일 대신 소스 파일에서 직접 import한다. **Barrel 파일**은 여러 모듈을 다시 export하는 진입점 파일이다 (예: `export * from './module'`을 하는 `index.js`).

인기 있는 아이콘·컴포넌트 라이브러리는 진입점 파일에 **최대 10,000개의 re-export**가 들어 있을 수 있다. 다수의 React 패키지는 **import만으로 200-800ms가 소요되어** 개발 속도와 프로덕션 cold start에 모두 영향을 준다.

**Why tree-shaking doesn't help:** 라이브러리가 external로 표시되면(번들에 포함되지 않으면) 번들러가 최적화를 할 수 없다. tree-shaking을 위해 번들에 포함시키면 전체 모듈 그래프 분석 때문에 빌드가 크게 느려진다.

**Incorrect (imports entire library):**

```tsx
import { Check, X, Menu } from 'lucide-react'
// Loads 1,583 modules, takes ~2.8s extra in dev
// Runtime cost: 200-800ms on every cold start

import { Button, TextField } from '@mui/material'
// Loads 2,225 modules, takes ~4.2s extra in dev
```

**Correct - Next.js 13.5+ (recommended):**

```js
// next.config.js - automatically optimizes barrel imports at build time
module.exports = {
  experimental: {
    optimizePackageImports: ['lucide-react', '@mui/material']
  }
}
```

```tsx
// Keep the standard imports - Next.js transforms them to direct imports
import { Check, X, Menu } from 'lucide-react'
// Full TypeScript support, no manual path wrangling
```

이 방식이 권장된다. TypeScript 타입 안전성과 에디터 자동완성을 유지하면서 barrel import 비용을 제거할 수 있기 때문이다.

**Correct - Direct imports (non-Next.js projects):**

```tsx
import Button from '@mui/material/Button'
import TextField from '@mui/material/TextField'
// Loads only what you use
```

> **TypeScript warning:** 일부 라이브러리(특히 `lucide-react`)는 deep import 경로에 대해 `.d.ts`를 제공하지 않는다. `lucide-react/dist/esm/icons/check`를 import하면 implicit `any`로 해석되어 `strict`나 `noImplicitAny` 환경에서 에러가 발생한다. 가능하면 `optimizePackageImports`를 우선 사용하고, direct import 전에 라이브러리가 subpath 타입을 export하는지 확인한다.

이러한 최적화로 dev boot 15-70% 단축, 빌드 28% 단축, cold start 40% 단축, HMR 속도가 크게 향상된다.

자주 영향받는 라이브러리: `lucide-react`, `@mui/material`, `@mui/icons-material`, `@tabler/icons-react`, `react-icons`, `@headlessui/react`, `@radix-ui/react-*`, `lodash`, `ramda`, `date-fns`, `rxjs`, `react-use`.

Reference: [How we optimized package imports in Next.js](https://vercel.com/blog/how-we-optimized-package-imports-in-next-js)
