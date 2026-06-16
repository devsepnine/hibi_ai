---
name: vercel-react-native-skills
description: React Native/Expo best practices — list performance (FlashList), animations (Reanimated), navigation, native modules, and monorepo setup for performant mobile apps. 리액트 네이티브, Expo 앱, 모바일 성능 최적화, 네이티브 모듈.
keywords: [react-native, 리액트네이티브, expo, mobile, 모바일, native]
license: MIT
metadata:
  author: vercel
  version: '1.0.0'
---

# React Native Skills

React Native 및 Expo 애플리케이션을 위한 종합 베스트 프랙티스. 성능, 애니메이션, UI 패턴, 플랫폼별 최적화를 아우르는 여러 카테고리의 규칙을 담고 있다.

## 적용 시점

다음 상황에서 이 가이드를 참조한다:

- React Native 또는 Expo 앱을 만들 때
- 리스트 및 스크롤 성능 최적화
- Reanimated로 애니메이션 구현
- 이미지 및 미디어 작업
- 네이티브 모듈 또는 폰트 설정
- 네이티브 의존성을 가진 모노레포 프로젝트 구조화

## 우선순위별 규칙 카테고리

| Priority | Category         | Impact   | Prefix               |
| -------- | ---------------- | -------- | -------------------- |
| 1        | List Performance | CRITICAL | `list-performance-`  |
| 2        | Animation        | HIGH     | `animation-`         |
| 3        | Navigation       | HIGH     | `navigation-`        |
| 4        | UI Patterns      | HIGH     | `ui-`                |
| 5        | State Management | MEDIUM   | `react-state-`       |
| 6        | Rendering        | MEDIUM   | `rendering-`         |
| 7        | Monorepo         | MEDIUM   | `monorepo-`          |
| 8        | Configuration    | LOW      | `fonts-`, `imports-` |

## Quick Reference

### 1. List Performance (CRITICAL)

- `list-performance-virtualize` - 큰 리스트에는 FlashList를 사용한다
- `list-performance-item-memo` - 리스트 아이템 컴포넌트는 memoize 한다
- `list-performance-callbacks` - 콜백 참조를 안정화한다
- `list-performance-inline-objects` - 인라인 style 객체를 피한다
- `list-performance-function-references` - 함수는 render 바깥으로 추출한다
- `list-performance-images` - 리스트 안의 이미지를 최적화한다
- `list-performance-item-expensive` - 비싼 작업은 아이템 바깥으로 옮긴다
- `list-performance-item-types` - 이질적 리스트에는 item types를 쓴다

### 2. Animation (HIGH)

- `animation-gpu-properties` - transform과 opacity만 애니메이션한다
- `animation-derived-value` - 계산된 애니메이션에는 useDerivedValue를 쓴다
- `animation-gesture-detector-press` - Pressable 대신 Gesture.Tap을 사용한다

### 3. Navigation (HIGH)

- `navigation-native-navigators` - JS 네비게이터보다 native stack과 native tabs를 사용한다

### 4. UI Patterns (HIGH)

- `ui-expo-image` - 모든 이미지에 expo-image를 사용한다
- `ui-image-gallery` - 이미지 라이트박스는 Galeria를 사용한다
- `ui-pressable` - TouchableOpacity 대신 Pressable을 사용한다
- `ui-safe-area-scroll` - ScrollView의 safe area를 처리한다
- `ui-scrollview-content-inset` - 헤더에는 contentInset을 사용한다
- `ui-menus` - 네이티브 컨텍스트 메뉴를 사용한다
- `ui-native-modals` - 가능하면 네이티브 모달을 사용한다
- `ui-measure-views` - measure() 대신 onLayout을 사용한다
- `ui-styling` - StyleSheet.create 또는 Nativewind를 사용한다

### 5. State Management (MEDIUM)

- `react-state-minimize` - 상태 구독을 최소화한다
- `react-state-dispatcher` - 콜백에는 dispatcher 패턴을 쓴다
- `react-state-fallback` - 첫 렌더에 fallback을 보여준다
- `react-compiler-destructure-functions` - React Compiler를 위해 destructure 한다
- `react-compiler-reanimated-shared-values` - 컴파일러와 함께 shared value를 다룬다

### 6. Rendering (MEDIUM)

- `rendering-text-in-text-component` - 텍스트는 Text 컴포넌트로 감싼다
- `rendering-no-falsy-and` - 조건부 렌더에 falsy &&를 피한다

### 7. Monorepo (MEDIUM)

- `monorepo-native-deps-in-app` - 네이티브 의존성은 앱 패키지에 둔다
- `monorepo-single-dependency-versions` - 패키지 간 단일 버전을 유지한다

### 8. Configuration (LOW)

- `fonts-config-plugin` - 커스텀 폰트는 config plugin을 사용한다
- `imports-design-system-folder` - 디자인 시스템 import를 정리한다
- `js-hoist-intl` - Intl 객체 생성을 hoist 한다

## 사용 방법

자세한 설명과 코드 예제는 개별 규칙 파일을 읽는다:

```
rules/list-performance-virtualize.md
rules/animation-gpu-properties.md
```

각 규칙 파일에는 다음이 들어 있다:

- 왜 중요한지에 대한 간단한 설명
- 잘못된 코드 예제와 설명
- 올바른 코드 예제와 설명
- 추가 컨텍스트와 참고 자료

## 전체 통합 문서

모든 규칙을 펼친 완전한 가이드: `AGENTS.md`
