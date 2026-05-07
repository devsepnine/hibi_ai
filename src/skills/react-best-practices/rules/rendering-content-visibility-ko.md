---
title: CSS content-visibility for Long Lists
impact: HIGH
impactDescription: faster initial render
tags: rendering, css, content-visibility, long-lists
---

## 긴 리스트에 CSS content-visibility를 적용한다

`content-visibility: auto`를 적용해 화면 밖 요소의 렌더링을 지연시킨다.

**CSS:**

```css
.message-item {
  content-visibility: auto;
  contain-intrinsic-size: 0 80px;
}
```

**예제:**

```tsx
function MessageList({ messages }: { messages: Message[] }) {
  return (
    <div className="overflow-y-auto h-screen">
      {messages.map(msg => (
        <div key={msg.id} className="message-item">
          <Avatar user={msg.author} />
          <div>{msg.content}</div>
        </div>
      ))}
    </div>
  )
}
```

메시지가 1000개라면, 브라우저는 화면 밖에 있는 약 990개 항목의 layout/paint를 건너뛴다 (초기 렌더링이 약 10배 빨라진다).
