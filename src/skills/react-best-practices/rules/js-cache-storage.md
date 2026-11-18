---
title: Cache Storage API Calls
impact: LOW-MEDIUM
impactDescription: reduces expensive I/O
tags: javascript, localStorage, storage, caching, performance
---

## Cache Storage API Calls

Use a memory cache for browser preferences that may lag until another tab’s storage event arrives. Route writes, removals, and clears in this tab through these functions. Read storage directly if other code writes in the same tab or the caller needs the latest value immediately.

Call the accessors in a browser event handler or an effect after mount. Register listeners in the browser so server rendering can import the module.

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

Cookies can expire or change independently of storage events. Read the current cookie string on each call and split each name from its value at the first equals sign.

```typescript
function getCookie(name: string) {
  const prefix = name + '='
  const pair = document.cookie.split(/;\s*/).find(entry => entry.startsWith(prefix))
  return pair?.slice(prefix.length)
}
```
