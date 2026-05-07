---
title: Use Passive Event Listeners for Scrolling Performance
impact: MEDIUM
impactDescription: eliminates scroll delay caused by event listeners
tags: client, event-listeners, scrolling, performance, touch, wheel
---

## Use Passive Event Listeners for Scrolling Performance

touch와 wheel 이벤트 리스너에 `{ passive: true }`를 붙여 즉시 스크롤되도록 한다. 브라우저는 보통 리스너가 끝나기를 기다려 `preventDefault()` 호출 여부를 확인하는데, 이로 인해 스크롤 지연이 생긴다.

**Incorrect:**

```typescript
useEffect(() => {
  const handleTouch = (e: TouchEvent) => console.log(e.touches[0].clientX)
  const handleWheel = (e: WheelEvent) => console.log(e.deltaY)
  
  document.addEventListener('touchstart', handleTouch)
  document.addEventListener('wheel', handleWheel)
  
  return () => {
    document.removeEventListener('touchstart', handleTouch)
    document.removeEventListener('wheel', handleWheel)
  }
}, [])
```

**Correct:**

```typescript
useEffect(() => {
  const handleTouch = (e: TouchEvent) => console.log(e.touches[0].clientX)
  const handleWheel = (e: WheelEvent) => console.log(e.deltaY)
  
  document.addEventListener('touchstart', handleTouch, { passive: true })
  document.addEventListener('wheel', handleWheel, { passive: true })
  
  return () => {
    document.removeEventListener('touchstart', handleTouch)
    document.removeEventListener('wheel', handleWheel)
  }
}, [])
```

**Use passive when:** tracking/analytics, logging 등 `preventDefault()`를 호출하지 않는 모든 리스너.

**Don't use passive when:** custom swipe 제스처 구현, custom zoom 컨트롤, 또는 `preventDefault()`가 필요한 모든 리스너.
