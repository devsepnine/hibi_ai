# React Native Guidelines

에이전트와 LLM에 최적화된 React Native 베스트 프랙티스를 작성하고 유지하기 위한 구조화된 저장소다.

## Structure

- `rules/` - 개별 규칙 파일 (규칙당 하나의 파일)
  - `_sections.md` - 섹션 메타데이터 (제목, 영향도, 설명)
  - `_template.md` - 새 규칙 작성용 템플릿
  - `area-description.md` - 개별 규칙 파일
- `metadata.json` - 문서 메타데이터 (버전, 조직, 초록)
- **`AGENTS.md`** - 컴파일된 출력 (자동 생성)

## Rules

### Core Rendering (CRITICAL)

- `rendering-text-in-text-component.md` - 문자열을 Text 컴포넌트로 감쌀 것
- `rendering-no-falsy-and.md` - JSX에서 falsy && 연산자를 피할 것

### List Performance (HIGH)

- `list-performance-virtualize.md` - 가상화 리스트(LegendList,
  FlashList) 사용
- `list-performance-function-references.md` - 안정적인 객체 참조 유지
- `list-performance-callbacks.md` - 콜백을 리스트 루트로 호이스팅
- `list-performance-inline-objects.md` - renderItem에서 인라인 객체 회피
- `list-performance-item-memo.md` - 메모이제이션을 위해 primitive 전달
- `list-performance-item-expensive.md` - 리스트 아이템을 가볍게 유지
- `list-performance-images.md` - 리스트에서 압축 이미지 사용
- `list-performance-item-types.md` - 이질 리스트에 item type 사용

### Animation (HIGH)

- `animation-gpu-properties.md` - 레이아웃 대신 transform/opacity 애니메이션
- `animation-gesture-detector-press.md` - 프레스 애니메이션에 GestureDetector
  사용
- `animation-derived-value.md` - useAnimatedReaction 대신 useDerivedValue 선호

### Scroll Performance (HIGH)

- `scroll-position-no-state.md` - 스크롤 위치를 useState로 추적하지 말 것

### Navigation (HIGH)

- `navigation-native-navigators.md` - native stack과 native tabs 사용

### React State (MEDIUM)

- `react-state-dispatcher.md` - 함수형 setState 업데이트 사용
- `react-state-fallback.md` - state는 사용자 의도만 표현
- `react-state-minimize.md` - state 변수 최소화, 값은 derive

### State Architecture (MEDIUM)

- `state-ground-truth.md` - state는 ground truth를 표현해야 함

### React Compiler (MEDIUM)

- `react-compiler-destructure-functions.md` - 함수를 일찍 destructure
- `react-compiler-reanimated-shared-values.md` - shared value에 .get()/.set()
  사용

### User Interface (MEDIUM)

- `ui-expo-image.md` - 최적화된 이미지를 위해 expo-image 사용
- `ui-image-gallery.md` - lightbox/갤러리에 Galeria 사용
- `ui-menus.md` - Zeego로 native dropdown과 context menu 구현
- `ui-native-modals.md` - formSheet과 함께 native Modal 사용
- `ui-pressable.md` - TouchableOpacity 대신 Pressable 사용
- `ui-measure-views.md` - view 치수 측정
- `ui-safe-area-scroll.md` - contentInsetAdjustmentBehavior 사용
- `ui-scrollview-content-inset.md` - 동적 spacing에 contentInset 사용
- `ui-styling.md` - 모던 스타일링 패턴 (gap, boxShadow, gradient)

### Design System (MEDIUM)

- `design-system-compound-components.md` - compound component 사용

### Monorepo (LOW)

- `monorepo-native-deps-in-app.md` - 네이티브 의존성을 앱 디렉토리에 설치
- `monorepo-single-dependency-versions.md` - 단일 의존성 버전 사용

### Third-Party Dependencies (LOW)

- `imports-design-system-folder.md` - 디자인 시스템 폴더에서 import

### JavaScript (LOW)

- `js-hoist-intl.md` - Intl formatter 생성을 호이스팅

### Fonts (LOW)

- `fonts-config-plugin.md` - 빌드 시점에 폰트를 native로 로드

## Creating a New Rule

1. `rules/_template.md`를 `rules/area-description.md`로 복사한다
2. 적절한 area prefix를 선택한다:
   - Core Rendering: `rendering-`
   - List Performance: `list-performance-`
   - Animation: `animation-`
   - Scroll Performance: `scroll-`
   - Navigation: `navigation-`
   - React State: `react-state-`
   - State Architecture: `state-`
   - React Compiler: `react-compiler-`
   - User Interface: `ui-`
   - Design System: `design-system-`
   - Monorepo: `monorepo-`
   - Third-Party Dependencies: `imports-`
   - JavaScript: `js-`
   - Fonts: `fonts-`
3. frontmatter와 본문을 채운다
4. 명확한 설명이 포함된 예시를 확보한다

## Rule File Structure

각 규칙 파일은 다음 구조를 따른다:

````markdown
---
title: Rule Title Here
impact: MEDIUM
impactDescription: Optional description
tags: tag1, tag2, tag3
---

## Rule Title Here

규칙에 대한 간략한 설명과 그것이 중요한 이유.

**Incorrect (description of what's wrong):**

```tsx
// Bad code example
```
````

**Correct (description of what's right):**

```tsx
// Good code example
```

Reference: [Link](https://example.com)

```

## File Naming Convention

- `_`로 시작하는 파일은 특별 파일이다 (빌드에서 제외)
- 규칙 파일: `area-description.md` (예: `animation-gpu-properties.md`)
- 섹션은 파일명 prefix로부터 자동으로 추론된다
- 규칙은 각 섹션 내에서 제목 알파벳 순으로 정렬된다

## Impact Levels

- `CRITICAL` - 최우선, 크래시 또는 깨진 UI를 야기
- `HIGH` - 상당한 성능 향상
- `MEDIUM` - 중간 정도의 성능 향상
- `LOW` - 점진적 향상
```
