# Migrating from Svelte 4 to Svelte 5

Svelte 5는 `.svelte` 파일과 대부분의 template 문법을 유지하고, backwards compatibility를 위해 v4 패턴을 인식하는 **legacy mode**를 제공한다. 새 코드는 runes mode (기본)를 사용해야 한다. 이 문서는 기계적인 변환과 맹목적인 swap이 잘못되는 미묘한 경우를 다룬다.

## Quick reference

| v4 | v5 |
|----|----|
| `export let name` | `let { name } = $props()` |
| `export let count = 0` | `let { count = 0 } = $props()` |
| `let x = 0`<br>`$: y = x * 2` | `let x = $state(0)`<br>`const y = $derived(x * 2)` |
| `$: if (x > 10) warn()` | `$effect(() => { if (x > 10) warn() })` |
| `$: { /* multi-line */ }` | `$effect(() => { /* multi-line */ })` |
| `on:click={fn}` | `onclick={fn}` |
| `on:click\|preventDefault={fn}` | `onclick={(e) => { e.preventDefault(); fn(e) }}` |
| `createEventDispatcher(); dispatch('save', x)` | `onsave: (x) => void` callback prop |
| `<slot />` | `{@render children?.()}` + `children?: Snippet` prop |
| `<slot name="header" />` | `{@render header?.()}` + `header?: Snippet` prop |
| `<slot {item} />` inside `{#each}` | argumented snippet 전달: `row: Snippet<[Item]>` |
| `<svelte:component this={Comp} />` | `<Comp />` (component는 값) |
| `writable(0)` in `.ts` | `let v = $state(0)` in `.svelte.ts` |
| `$store` auto-subscribe | `store.value` (runed store 사용 시) |
| `bind:this={el}` | 동일 — 변경 없음 |
| `<svelte:self />` | 동일 — 변경 없음 |

## Props

### v4

```svelte
<script>
  export let title
  export let count = 0
</script>
```

### v5

```svelte
<script lang="ts">
  type Props = { title: string; count?: number }
  let { title, count = 0 }: Props = $props()
</script>
```

주의 사항:

- TypeScript 타입은 JSDoc이 아닌 destructure annotation에 들어간다.
- 기본값은 일반 JS destructure 기본값이다.
- props는 **기본적으로 reactive**이다 — read는 track되며 `$:`가 필요 없다.

### Forwarding props

v4:
```svelte
<input {...$$restProps} />
```

v5:
```svelte
<script lang="ts">
  let { onclick, ...rest } = $props()
</script>
<input {...rest} onclick={onclick} />
```

## Reactive declarations (`$:`)

v4의 `$:`는 두 가지를 했고 runes는 이를 분리한다:

- **계산된 값** → `$derived`
- **사이드 이펙트** → `$effect`

### v4 (mixed)

```svelte
<script>
  let count = 0
  $: doubled = count * 2                           // computation
  $: console.log('count changed:', count)          // side effect
  $: if (count > 10) count = 0                     // side effect that writes
</script>
```

### v5 (explicit)

```svelte
<script>
  let count = $state(0)
  const doubled = $derived(count * 2)
  $effect(() => { console.log('count changed:', count) })

  // Don't write to a dependency inside $effect — lift to the event handler
  function increment() {
    count = count >= 10 ? 0 : count + 1
  }
</script>
```

세 번째 경우 (`$:` 내부에서 dep에 write)는 v4에서 미묘하게 깨졌으며 v5에서는 명시적으로 패턴 냄새이다 — 기계적으로 `$effect`로 옮기면 무한 루프가 된다. 조건을 업데이트를 트리거한 이벤트로 옮긴다.

## Events

v4는 modifier 단축형이 있는 event directive (`on:click`)를 사용했다. v5는 일반 attribute를 사용한다.

### v4

```svelte
<button on:click={handle} on:click|preventDefault={handle2} />
```

### v5

```svelte
<button onclick={handle} />
<button onclick={(e) => { e.preventDefault(); handle2(e) }} />
```

Modifier (`|preventDefault`, `|stopPropagation`, `|once`, `|capture`, `|self`, `|trusted`, `|nonpassive`, `|passive`)는 v5에 없다. 일반 JS로 교체:

| v4 modifier | v5 equivalent |
|-------------|---------------|
| `preventDefault` | 상단에서 `e.preventDefault()` 호출 |
| `stopPropagation` | `e.stopPropagation()` 호출 |
| `once` | 첫 호출 후 unsubscribe하도록 핸들러 wrap |
| `capture` | `addEventListener('click', h, true)`로 수동 추가 (또는 attachment 사용) |
| `self` | `e.target === e.currentTarget` 확인 |
| `trusted` | `e.isTrusted` 확인 |
| `passive` / `nonpassive` | 명시적 `addEventListener(..., { passive: ... })`로 `@attach` 사용 |

## Event dispatchers → callback props

v4:
```svelte
<!-- Counter.svelte -->
<script>
  import { createEventDispatcher } from 'svelte'
  const dispatch = createEventDispatcher<{ change: number }>()
  let n = 0
  function inc() { n++; dispatch('change', n) }
</script>
<button on:click={inc}>{n}</button>

<!-- parent -->
<Counter on:change={(e) => console.log(e.detail)} />
```

v5:
```svelte
<!-- Counter.svelte -->
<script lang="ts">
  let { onchange }: { onchange?: (n: number) => void } = $props()
  let n = $state(0)
  function inc() { n++; onchange?.(n) }
</script>
<button onclick={inc}>{n}</button>

<!-- parent -->
<Counter onchange={(n) => console.log(n)} />
```

자식이 callback을 직접 호출한다 — `.detail` 없음, `CustomEvent` 없음. 타입이 prop을 통해 자연스럽게 흐른다.

## Slots → snippets

### Default slot

v4:
```svelte
<!-- Card.svelte -->
<div class="card"><slot /></div>

<!-- parent -->
<Card>Hello</Card>
```

v5:
```svelte
<!-- Card.svelte -->
<script lang="ts">
  import type { Snippet } from 'svelte'
  let { children }: { children?: Snippet } = $props()
</script>
<div class="card">{@render children?.()}</div>

<!-- parent (unchanged syntax) -->
<Card>Hello</Card>
```

### Slots with props

v4:
```svelte
<!-- List.svelte -->
<ul>
  {#each items as item}
    <li><slot {item} /></li>
  {/each}
</ul>

<!-- parent -->
<List {items} let:item>
  <a href={item.url}>{item.name}</a>
</List>
```

v5:
```svelte
<!-- List.svelte -->
<script lang="ts">
  import type { Snippet } from 'svelte'
  type Props<T> = { items: T[]; row: Snippet<[T]> }
  let { items, row }: Props<{ url: string; name: string }> = $props()
</script>
<ul>
  {#each items as item}
    <li>{@render row(item)}</li>
  {/each}
</ul>

<!-- parent -->
<List {items}>
  {#snippet row(item)}
    <a href={item.url}>{item.name}</a>
  {/snippet}
</List>
```

Snippet은 first-class 값이다 — 여러 개를 전달, prop으로 받기, 인자에 타입 부여 가능. slot보다 강력하다 (오래된 `let:` 문법은 사라짐).

### Named slots

v4의 `<slot name="header" />` → 타입 `Snippet`인 named prop, `{@render header?.()}`로 렌더링. 부모는 component 태그 내부에 `{#snippet header()}...{/snippet}`을 작성한다.

## Stores

`svelte/store` API는 여전히 작동한다 — 그러나 새 코드는 `.svelte.ts` 모듈의 runes를 선호해야 한다.

### v4

```ts
// cart.ts
import { writable, derived } from 'svelte/store'
export const items = writable<Item[]>([])
export const total = derived(items, ($items) =>
  $items.reduce((n, it) => n + it.qty, 0),
)
```

```svelte
<script>
  import { items, total } from './cart'
</script>
<p>Total: {$total}</p>   <!-- $ prefix for auto-subscribe -->
```

### v5

```ts
// cart.svelte.ts
let items = $state<Item[]>([])
const total = $derived(items.reduce((n, it) => n + it.qty, 0))

export const cart = {
  get items() { return items },
  get total() { return total },
  add(it: Item) { items.push(it) },
}
```

```svelte
<script>
  import { cart } from './cart.svelte'
</script>
<p>Total: {cart.total}</p>   <!-- no $ prefix -->
```

이점:

- subscribe/unsubscribe 춤 없음.
- class와 object에서 작동 (getter로 노출).
- TypeScript 타입이 `Writable<T>` wrapper 없이 흐른다.

v4 store를 유지하면 (점진적 마이그레이션), `.svelte` 파일에서 여전히 `$` prefix로 자동 subscribe된다.

## Dynamic components

v4:
```svelte
<svelte:component this={Component} {...props} />
```

v5:
```svelte
<Component {...props} />
```

Component는 v5에서 평범한 값이다 — 변수에 할당, prop으로 전달, `$derived`에서 선택. `<svelte:component>`는 여전히 컴파일된다 (legacy mode) 그러나 불필요하다.

## Mixed mode

Svelte 5는 v4 스타일 파일을 runes-mode 파일과 함께 실행할 수 있다. 중요한 규칙:

- **한 파일에 섞지 마라.** 단일 component는 legacy 또는 runes 중 하나이다 — `export let`도 있는 파일에서 `$state`를 사용하면 컴파일에 실패한다.
- **runes mode는 파일별 opt-in** — 어떤 rune (`$state`, `$props` 등)을 작성하면 파일이 flip된다. 그렇지 않으면 컴파일러가 v4 의미를 유지한다.
- heuristic이 실패하면 (드물게) `<svelte:options runes={true} />` 또는 `<svelte:options runes={false} />`를 **명시적으로 설정**할 수 있다.

큰 앱의 권장 마이그레이션 경로:

1. 코드 변경 없이 Svelte 5로 업그레이드 — legacy mode가 작동하게 한다.
2. 먼저 leaf component를 마이그레이션한다 (의존성이 가장 적음).
3. 공유 state는 `.svelte.ts`로 옮긴다; store 버전을 삭제한다.
4. 레이아웃과 라우트는 마지막에 마이그레이션한다.
5. 모든 것이 runes가 되면 설정했을 수 있는 `"svelteOptions.legacy"` 컴파일러 플래그를 삭제한다.

## Automated migration

Svelte는 `sv migrate svelte-5`를 제공한다:

```bash
npx sv migrate svelte-5
```

이는 ambiguous한 경우에 prompt와 함께 파일별로 기계적인 변환 (props, `$:`, `on:event`, `<slot>`)을 처리한다. 브랜치에서 실행하고, diff를 검토하고, 유지할 것을 선택하는 것이 안전하다.

store를 `.svelte.ts` 모듈로 변환하지 **않으며** (너무 프로젝트별), 관심사를 섞은 component를 재구조화하지 않는다 — 그것들은 사람이 직접 처리하도록 남겨둔다.

## Gotchas worth calling out

- **Legacy mode는 runes를 주지 않는다.** v5 프로젝트에서 열린 v4 파일은 나머지 앱이 v5 컴파일러 하에 실행 중이라도 여전히 `$:` reactivity를 사용한다. "부분적으로 업그레이드"하기를 바라며 `$state`를 뿌리지 마라.
- **`$$props` / `$$restProps`는 legacy 전용이다.** runes mode에서는 `let { ...rest } = $props()`을 사용한다.
- **DOM element의 `on:event`**는 legacy mode에서 여전히 컴파일된다 — 그러나 runes mode에서는 일반 attribute로 전환한다. `on:*` 타입은 DOM attribute 타입보다 덜 정확하다.
- **`$app/stores`**는 작동하지만 SvelteKit 2에서 deprecation을 로깅한다. `$app/state`로 마이그레이션한다 (`$` prefix 없음, plain reactive 액세스).
- **`beforeNavigate` / `afterNavigate` lifecycle hook**은 `$app/navigation`으로 이동했다 — `svelte`에서 import하고 있다면 멈춰라.
