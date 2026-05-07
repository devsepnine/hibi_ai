---
title: Prevent Hydration Mismatch Without Flickering
impact: MEDIUM
impactDescription: avoids visual flicker and hydration errors
tags: rendering, ssr, hydration, localStorage, flicker
---

## 깜빡임 없이 Hydration 불일치를 방지한다

클라이언트 측 저장소(localStorage, cookies)에 의존하는 콘텐츠를 렌더링할 때, SSR 깨짐과 hydration 이후 깜빡임을 모두 피하려면 React가 hydrate하기 전에 DOM을 갱신하는 동기 스크립트를 주입한다.

**잘못된 예 (SSR이 깨짐):**

```tsx
function ThemeWrapper({ children }: { children: ReactNode }) {
  // localStorage is not available on server - throws error
  const theme = localStorage.getItem('theme') || 'light'

  return (
    <div className={theme}>
      {children}
    </div>
  )
}
```

`localStorage`가 정의되지 않아 서버 측 렌더링에서 실패한다.

**잘못된 예 (시각적 깜빡임 발생):**

```tsx
function ThemeWrapper({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState('light')

  useEffect(() => {
    // Runs after hydration - causes visible flash
    const stored = localStorage.getItem('theme')
    if (stored) {
      setTheme(stored)
    }
  }, [])

  return (
    <div className={theme}>
      {children}
    </div>
  )
}
```

컴포넌트가 먼저 기본값(`light`)으로 렌더되고, hydration 이후 갱신되어 잘못된 콘텐츠가 잠깐 보이는 깜빡임이 발생한다.

**올바른 예 (깜빡임 없음, hydration 불일치 없음):**

```tsx
function ThemeWrapper({ children }: { children: ReactNode }) {
  const inlineScript = `
    (function() {
      try {
        var theme = localStorage.getItem('theme') || 'light';
        var el = document.getElementById('theme-wrapper');
        if (el) el.className = theme;
      } catch (e) {}
    })();
  `
  return (
    <>
      <div id="theme-wrapper">
        {children}
      </div>
      <script dangerouslySetInnerHTML={{ __html: inlineScript }} />
    </>
  )
}
```

인라인 스크립트는 요소가 표시되기 전에 동기적으로 실행되므로 DOM에 이미 올바른 값이 적용된 상태가 된다. 깜빡임도, hydration 불일치도 없다.

이 패턴은 테마 토글, 사용자 환경설정, 인증 상태 등 기본값을 깜빡이지 않고 즉시 렌더해야 하는 모든 클라이언트 전용 데이터에 특히 유용하다.
