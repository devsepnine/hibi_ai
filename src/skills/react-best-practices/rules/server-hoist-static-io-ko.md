---
title: Hoist Static I/O to Module Level
impact: HIGH
impactDescription: avoids repeated file/network I/O per request
tags: server, io, performance, next.js, route-handlers, og-image
---

## 정적 I/O를 모듈 레벨로 호이스트한다

**Impact: HIGH (요청마다 반복되는 파일/네트워크 I/O를 회피)**

route handler나 server function에서 정적 자산(폰트, 로고, 이미지, 설정 파일)을 로딩할 때는 I/O 연산을 모듈 레벨로 호이스트한다. 모듈 레벨 코드는 모듈이 처음 import될 때 한 번만 실행되며, 매 요청에서 실행되지 않는다. 이로써 호출마다 발생하는 중복된 파일 시스템 읽기나 네트워크 fetch를 제거한다.

**잘못된 예 (매 요청마다 폰트 파일을 읽음):**

```typescript
// app/api/og/route.tsx
import { ImageResponse } from 'next/og'

export async function GET(request: Request) {
  // Runs on EVERY request - expensive!
  const fontData = await fetch(
    new URL('./fonts/Inter.ttf', import.meta.url)
  ).then(res => res.arrayBuffer())

  const logoData = await fetch(
    new URL('./images/logo.png', import.meta.url)
  ).then(res => res.arrayBuffer())

  return new ImageResponse(
    <div style={{ fontFamily: 'Inter' }}>
      <img src={logoData} />
      Hello World
    </div>,
    { fonts: [{ name: 'Inter', data: fontData }] }
  )
}
```

**올바른 예 (모듈 초기화 시 한 번만 로드):**

```typescript
// app/api/og/route.tsx
import { ImageResponse } from 'next/og'

// Module-level: runs ONCE when module is first imported
const fontData = fetch(
  new URL('./fonts/Inter.ttf', import.meta.url)
).then(res => res.arrayBuffer())

const logoData = fetch(
  new URL('./images/logo.png', import.meta.url)
).then(res => res.arrayBuffer())

export async function GET(request: Request) {
  // Await the already-started promises
  const [font, logo] = await Promise.all([fontData, logoData])

  return new ImageResponse(
    <div style={{ fontFamily: 'Inter' }}>
      <img src={logo} />
      Hello World
    </div>,
    { fonts: [{ name: 'Inter', data: font }] }
  )
}
```

**올바른 예 (모듈 레벨에서 동기 fs):**

```typescript
// app/api/og/route.tsx
import { ImageResponse } from 'next/og'
import { readFileSync } from 'fs'
import { join } from 'path'

// Synchronous read at module level - blocks only during module init
const fontData = readFileSync(
  join(process.cwd(), 'public/fonts/Inter.ttf')
)

const logoData = readFileSync(
  join(process.cwd(), 'public/images/logo.png')
)

export async function GET(request: Request) {
  return new ImageResponse(
    <div style={{ fontFamily: 'Inter' }}>
      <img src={logoData} />
      Hello World
    </div>,
    { fonts: [{ name: 'Inter', data: fontData }] }
  )
}
```

**잘못된 예 (호출마다 config 읽기):**

```typescript
import fs from 'node:fs/promises'

export async function processRequest(data: Data) {
  const config = JSON.parse(
    await fs.readFile('./config.json', 'utf-8')
  )
  const template = await fs.readFile('./template.html', 'utf-8')

  return render(template, data, config)
}
```

**올바른 예 (config와 template을 모듈 레벨로 호이스트):**

```typescript
import fs from 'node:fs/promises'

const configPromise = fs
  .readFile('./config.json', 'utf-8')
  .then(JSON.parse)
const templatePromise = fs.readFile('./template.html', 'utf-8')

export async function processRequest(data: Data) {
  const [config, template] = await Promise.all([
    configPromise,
    templatePromise,
  ])

  return render(template, data, config)
}
```

이 패턴을 사용하는 시점:

- OG image 생성을 위한 폰트 로딩
- 정적 로고, 아이콘, 워터마크 로딩
- 런타임에 변하지 않는 설정 파일 읽기
- 이메일 템플릿 등 정적 템플릿 로딩
- 모든 요청에서 동일한 정적 자산

이 패턴을 사용하지 않는 시점:

- 요청 또는 사용자별로 달라지는 자산
- 런타임 도중 변경 가능한 파일 (대신 TTL이 있는 캐싱 사용)
- 유지하면 메모리를 과도하게 소비할 큰 파일
- 메모리에 남아 있어서는 안 되는 민감 정보

Vercel의 [Fluid Compute](https://vercel.com/docs/fluid-compute)에서는 여러 동시 요청이 같은 함수 인스턴스를 공유하므로 모듈 레벨 캐싱이 특히 효과적이다. 정적 자산은 cold start 패널티 없이 메모리에 유지된다.

전통적인 서버리스에서는 cold start마다 모듈 레벨 코드가 다시 실행되지만, 후속 warm invocation은 인스턴스가 회수될 때까지 로드된 자산을 재사용한다.
