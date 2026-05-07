# Runes

Runes는 Svelte 5에서 reactivity를 구동하는 primitive이다. `$`로 prefix된 함수처럼 보이지만 (`$state`, `$derived`, `$effect`, `$props`, `$bindable`, `$inspect`), 컴파일 타임 키워드이다 — 컴파일러가 이를 기저 reactivity 머신으로 다시 작성한다.

mental model: **runes는 reactive 형태를 선언하고; 일반 JavaScript가 그것들을 구동한다**. rune에 절대 "subscribe"하지 않는다. rune의 값을 read하면, 컴파일러가 의존성을 추적한다. 그것에 write하면, read한 모든 것이 다시 실행된다.

## `$state` — the default reactive source

변경되고 그 변경이 UI를 업데이트해야 하는 모든 값에 `$state`를 사용한다.

```svelte
<script>
  let count = $state(0)
  let todos = $state<Todo[]>([])
</script>
```

`$state`는 **deep proxy**를 반환한다. 어떤 property의 read도 추적되며; 어떤 property에 write (중첩 포함)도 업데이트를 트리거한다.

```ts
let user = $state({ name: 'Ada', address: { city: 'London' } })
user.name = 'Grace'              // triggers
user.address.city = 'NYC'        // triggers (nested works)
user.address = { city: 'Paris' } // triggers
```

### `$state.raw` — shallow, no proxy

중첩 업데이트가 아니라 교체로 mutate하는 큰 데이터에 `$state.raw`를 적용한다. 전체 값을 항상 다시 할당할 뿐이라면 proxy 오버헤드는 낭비이다.

```ts
let rows = $state.raw<Row[]>([])   // no per-cell tracking
function setRows(next: Row[]) { rows = next }
```

raw와 non-raw 액세스를 **섞지 마라** — raw 배열의 `rows.push(...)`는 트리거하지 않는다. 대신 `rows = [...rows, item]`을 사용하거나 `raw`를 drop한다.

### Class fields can be reactive too

```ts
class Counter {
  value = $state(0)
  step = $state(1)
  increment() { this.value += this.step }
}
```

각 `$state` field는 인스턴스별로 reactive이다. class는 여전히 class처럼 동작한다 (`new Counter()`, `extends` 등).

### Gotchas

- **state proxy 반복**은 reactive read를 생성한다; `$derived`에서 반복하면 배열이 변경될 때 derived가 다시 실행된다. 보통 원하는 것이지만 — 인지하라.
- **`JSON.stringify(state)`**는 모든 property를 read하므로 caller가 전체 트리에 의존하게 만든다. 로깅에는 괜찮지만; hot `$derived`에서는 피하라.
- **`structuredClone(state)`**는 non-reactive 복사본을 준다. optimistic UI 패턴을 위한 스냅샷을 원할 때 유용하다.

## `$derived` — computed values

`$derived(expr)`은 의존성이 변경될 때 다시 평가되는 순수 표현이다.

```ts
let count = $state(0)
const doubled = $derived(count * 2)
const label  = $derived(`Count: ${count}`)
```

규칙:

- 표현은 **순수**해야 한다 — I/O 없음, state write 없음, `console.log` 없음. 컴파일러가 마음대로 다시 실행한다.
- 표현 내부에서 read한 모든 reactive state가 의존성이 된다.
- 의존성은 실행마다 발견된다; 조건부 read가 정확히 작동한다.

### `$derived.by(() => ...)` — function form

표현이 여러 statement나 early return이 필요할 때:

```ts
const tax = $derived.by(() => {
  if (country === 'US') return price * 0.07
  if (country === 'KR') return price * 0.10
  return 0
})
```

`$derived`와 같은 규칙 — 순수성은 협상 불가이다.

### Don't put effects in a derived

로깅, fetch, mutate하는 `$derived`는 버그이다. 그것을 `$effect`나 트리거 이벤트 핸들러로 옮긴다.

## `$effect` — side effects

`$effect(fn)`는 component가 mount된 후, 그리고 read한 reactive 값이 변경될 때마다 `fn`을 실행한다.

```ts
$effect(() => {
  // dependency: `selectedId`. Runs after mount + on every change.
  localStorage.setItem('selected', String(selectedId))
})

$effect(() => {
  const socket = new WebSocket(url)
  return () => socket.close()        // cleanup — runs on change + unmount
})
```

cleanup은 반환 값이다. 정리할 것이 없으면 `undefined`를 반환한다.

### `$effect.pre` — before DOM commit

드물지만 layout read (페인트 전 측정)에 필요하다:

```ts
$effect.pre(() => {
  const rect = el.getBoundingClientRect()
  // ...
})
```

대부분의 effect는 일반 `$effect`여야 한다.

### `$effect.root` — imperative lifetimes

component 외부에서 소유된 effect (예: 한 번 초기화되는 `.svelte.ts` 모듈):

```ts
const destroy = $effect.root(() => {
  $effect(() => { /* ... */ })
  return () => { /* root teardown */ }
})
```

완료 시 `destroy()`를 호출한다. component 경계 밖에서 runes가 실행될 때 필요하다 — 그렇지 않으면 컴파일러가 effect가 언제 멈춰야 하는지 알 수 없다.

### Golden rule

**`$effect`는 외부 세계를 위한 것이다**. DOM API, 타이머, subscription, 분석, 로깅. 자신의 state 전환을 위한 re-run hook이 **아니다**.

```ts
// ❌ loop waiting to happen
$effect(() => {
  if (count > 10) count = 0     // effect writes to its own dependency
})

// ✅ put the logic where the change originates
function increment() {
  count = count >= 10 ? 0 : count + 1
}
```

## `$props` — component inputs

```svelte
<script lang="ts">
  type Props = {
    title: string
    count?: number
    children?: Snippet
  }
  let { title, count = 0, children }: Props = $props()
</script>
```

- 항상 destructure한다. 전달할 "props object"가 없다.
- 기본값은 type이 아닌 destructure에 들어간다.
- 타이핑은 일반 TS destructure annotation이다 — 특수 문법 없음.

### Renaming, rest, forwarding

```ts
// Rename: "class" is a JS keyword
let { class: className, ...rest } = $props()

// Forward remaining props to an inner element
<button class={className} {...rest}>{title}</button>
```

### Reading without destructuring

`$props()`는 proxy 같은 객체를 반환한다. field에 이름을 주지 않고 전체 묶음이 정말로 필요하다면, 전체로 잡을 수 있다:

```ts
const all = $props()
console.log(all.title, all.count)
```

거의 유용하지 않다. 한눈에 props가 보이도록 destructure를 선호한다.

## `$bindable` — opt-in two-way binding

평범한 prop은 read-only이다. 부모가 `bind:value`로 양방향 바인딩하게 하려면 `$bindable`로 opt-in한다:

```svelte
<!-- TextField.svelte -->
<script>
  let { value = $bindable('') } = $props()
</script>
<input bind:value />

<!-- parent -->
<TextField bind:value={formState.name} />
```

기본값 (`$bindable`의 첫 인자)은 부모가 bind하지 않을 때 사용된다. `$bindable` 없이는 이 component의 부모 `bind:value={...}`는 컴파일 에러이다 — 좋다, 대부분의 prop은 양방향이어서는 안 된다.

## `$inspect` — dev-only logging

```ts
$inspect(count, doubled)             // logs every time they change
$inspect('search:', query).with((type, value) => {
  if (type === 'update') sendMetric(value)
})
```

- production 빌드에서 strip된다.
- `.with`는 커스텀 핸들러를 준다 (initial vs update event).
- reactivity 디버깅에 좋다 — re-run을 기대하는데 발생하지 않을 때, `$inspect`는 값이 실제로 변경되었는지 알려준다.

## `$host` — custom elements only

`<svelte:options customElement="...">` component 내부에서 `$host()`는 호스팅 custom element를 반환한다. DOM 이벤트 dispatch, 브라우저가 설정한 attribute read 등. 일반 Svelte 앱에는 관련 없다.

## Decision tree

- "변경되는 값이 있다" → `$state`
- "다른 것에서 계산된 값이 있다" → `$derived` (또는 `$derived.by`)
- "값이 변경될 때 무언가 해야 한다 (Svelte 외부)" → `$effect`
- "component를 작성 중이다" → `$props` (매번)
- "내 component에서 `bind:value`를 원한다" → `$bindable`
- "왜 이것이 다시 실행되었나/안 되었나?" → `$inspect`
