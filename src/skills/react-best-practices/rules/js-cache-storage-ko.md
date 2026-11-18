---
title: Cache Storage API Calls
impact: LOW-MEDIUM
impactDescription: reduces expensive I/O
tags: javascript, localStorage, storage, caching, performance
---

## Cache Storage API Calls

다른 탭의 변경 알림이 도착할 때까지 예전 값을 보여도 되는 브라우저 설정에 메모리 캐시를 사용한다. 같은 탭의 쓰기·삭제·초기화는 아래 함수로 처리한다. 같은 탭에서 다른 코드가 직접 수정하거나 즉시 최신 값이 필요하면 저장소에서 직접 읽는다.

브라우저 이벤트 핸들러나 마운트 후 실행되는 effect에서 이 함수를 호출한다. 서버 렌더링 중에도 모듈을 불러올 수 있도록 리스너는 브라우저에서 등록한다.

```typescript
const storageCache = new Map<string, string | null>()

function getLocalStorage(key: string): string | null {
  if (!storageCache.has(key)) storageCache.set(key, localStorage.getItem(key))
  return storageCache.get(key) ?? null
}

function setLocalStorage(key: string, value: string) {
  localStorage.setItem(key, value)
  storageCache.set(key, value)
}

function removeLocalStorage(key: string) {
  localStorage.removeItem(key)
  storageCache.delete(key)
}

function clearLocalStorage() {
  localStorage.clear()
  storageCache.clear()
}

if (typeof window !== 'undefined') {
  window.addEventListener('storage', (event) => {
    if (event.storageArea !== localStorage) return
    if (event.key === null) storageCache.clear()
    else storageCache.delete(event.key)
  })

  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') storageCache.clear()
  })
}
```

브라우저와 서버는 저장소 이벤트를 보내지 않고 쿠키를 변경하거나 만료시킬 수 있다. 읽을 때 현재 쿠키 문자열에서 이름 다음의 첫 `=`까지만 구분자로 사용한다.

```typescript
function getCookie(name: string) {
  const prefix = name + '='
  const pair = document.cookie.split(/;\s*/).find(entry => entry.startsWith(prefix))
  return pair?.slice(prefix.length)
}
```
