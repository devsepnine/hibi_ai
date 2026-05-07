---
title: Use useRef for Transient Values
impact: MEDIUM
impactDescription: avoids unnecessary re-renders on frequent updates
tags: rerender, useref, state, performance
---

## 일시적인 값에는 useRef를 사용한다

값이 자주 변하고 매 업데이트마다 재렌더를 원하지 않을 때(예: 마우스 트래커, 인터벌, 일시적 플래그), `useState` 대신 `useRef`에 저장한다. 컴포넌트 state는 UI를 위해 두고, ref는 일시적인 DOM 인접 값에 사용한다. ref 갱신은 재렌더를 트리거하지 않는다.

**잘못된 예 (매 업데이트마다 렌더):**

```tsx
function Tracker() {
  const [lastX, setLastX] = useState(0)

  useEffect(() => {
    const onMove = (e: MouseEvent) => setLastX(e.clientX)
    window.addEventListener('mousemove', onMove)
    return () => window.removeEventListener('mousemove', onMove)
  }, [])

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: lastX,
        width: 8,
        height: 8,
        background: 'black',
      }}
    />
  )
}
```

**올바른 예 (트래킹에 재렌더 없음):**

```tsx
function Tracker() {
  const lastXRef = useRef(0)
  const dotRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      lastXRef.current = e.clientX
      const node = dotRef.current
      if (node) {
        node.style.transform = `translateX(${e.clientX}px)`
      }
    }
    window.addEventListener('mousemove', onMove)
    return () => window.removeEventListener('mousemove', onMove)
  }, [])

  return (
    <div
      ref={dotRef}
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        width: 8,
        height: 8,
        background: 'black',
        transform: 'translateX(0px)',
      }}
    />
  )
}
```
