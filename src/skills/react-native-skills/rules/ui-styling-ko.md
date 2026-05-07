---
title: Modern React Native Styling Patterns
impact: MEDIUM
impactDescription: consistent design, smoother borders, cleaner layouts
tags: styling, css, layout, shadows, gradients
---

## Modern React Native Styling Patterns

더 깔끔하고 일관된 React Native 코드를 위해 다음 스타일링 패턴을 따른다.

**`borderRadius`를 쓸 때는 항상 `borderCurve: 'continuous'`도 같이 쓴다:**

```tsx
// Incorrect
{ borderRadius: 12 }

// Correct – smoother iOS-style corners
{ borderRadius: 12, borderCurve: 'continuous' }
```

**요소 사이 간격에는 margin이 아니라 `gap`을 사용한다:**

```tsx
// Incorrect – margin on children
<View>
  <Text style={{ marginBottom: 8 }}>Title</Text>
  <Text style={{ marginBottom: 8 }}>Subtitle</Text>
</View>

// Correct – gap on parent
<View style={{ gap: 8 }}>
  <Text>Title</Text>
  <Text>Subtitle</Text>
</View>
```

**내부 여백에는 `padding`, 요소 간 간격에는 `gap`을 쓴다:**

```tsx
<View style={{ padding: 16, gap: 12 }}>
  <Text>First</Text>
  <Text>Second</Text>
</View>
```

**linear gradient에는 `experimental_backgroundImage`를 쓴다:**

```tsx
// Incorrect – third-party gradient library
<LinearGradient colors={['#000', '#fff']} />

// Correct – native CSS gradient syntax
<View
  style={{
    experimental_backgroundImage: 'linear-gradient(to bottom, #000, #fff)',
  }}
/>
```

**그림자에는 CSS `boxShadow` 문자열 문법을 사용한다:**

```tsx
// Incorrect – legacy shadow objects or elevation
{ shadowColor: '#000', shadowOffset: { width: 0, height: 2 }, shadowOpacity: 0.1 }
{ elevation: 4 }

// Correct – CSS box-shadow syntax
{ boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)' }
```

**여러 폰트 사이즈를 사용하지 않고, 강조에는 weight와 color를 활용한다:**

```tsx
// Incorrect – varying font sizes for hierarchy
<Text style={{ fontSize: 18 }}>Title</Text>
<Text style={{ fontSize: 14 }}>Subtitle</Text>
<Text style={{ fontSize: 12 }}>Caption</Text>

// Correct – consistent size, vary weight and color
<Text style={{ fontWeight: '600' }}>Title</Text>
<Text style={{ color: '#666' }}>Subtitle</Text>
<Text style={{ color: '#999' }}>Caption</Text>
```

폰트 사이즈를 제한하면 시각적 일관성이 생긴다. 대신 `fontWeight`(bold/semibold)와
회색조 컬러로 위계를 표현한다.
