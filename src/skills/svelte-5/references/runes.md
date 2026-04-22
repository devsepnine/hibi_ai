# Runes

Runes are the primitives that power reactivity in Svelte 5. They look
like functions prefixed with `$` (`$state`, `$derived`, `$effect`,
`$props`, `$bindable`, `$inspect`) but they're compile-time keywords —
the compiler rewrites them into the underlying reactivity machinery.

The mental model: **runes declare reactive shapes; ordinary JavaScript
drives them**. You never "subscribe" to a rune. Read a rune's value,
and the compiler tracks that you depend on it. Write to it, and
anything that read it re-runs.

## `$state` — the default reactive source

Use `$state` for any value that changes and whose change should update
the UI.

```svelte
<script>
  let count = $state(0)
  let todos = $state<Todo[]>([])
</script>
```

`$state` returns a **deep proxy**. Reads of any property are tracked;
writes to any property (including nested ones) trigger updates.

```ts
let user = $state({ name: 'Ada', address: { city: 'London' } })
user.name = 'Grace'              // triggers
user.address.city = 'NYC'        // triggers (nested works)
user.address = { city: 'Paris' } // triggers
```

### `$state.raw` — shallow, no proxy

Apply `$state.raw` to large data you mutate by replacement, not by
nested update. The proxy overhead is wasted if you only ever reassign
the whole value.

```ts
let rows = $state.raw<Row[]>([])   // no per-cell tracking
function setRows(next: Row[]) { rows = next }
```

**Don't** mix raw and non-raw access — `rows.push(...)` on a raw
array won't trigger. Use `rows = [...rows, item]` instead, or drop
the `raw`.

### Class fields can be reactive too

```ts
class Counter {
  value = $state(0)
  step = $state(1)
  increment() { this.value += this.step }
}
```

Each `$state` field is reactive per-instance. The class still behaves
like a class (`new Counter()`, `extends`, etc.).

### Gotchas

- **Iterating a state proxy** produces reactive reads; if you iterate
  in a `$derived`, the derived re-runs when the array changes.
  Usually what you want — but be aware.
- **`JSON.stringify(state)`** reads every property and so makes the
  caller depend on the whole tree. Fine for logging; avoid in hot
  `$derived`s.
- **`structuredClone(state)`** gives you a non-reactive copy. Useful
  when you want a snapshot for optimistic UI patterns.

## `$derived` — computed values

`$derived(expr)` is a pure expression that re-evaluates when its
dependencies change.

```ts
let count = $state(0)
const doubled = $derived(count * 2)
const label  = $derived(`Count: ${count}`)
```

Rules:

- The expression must be **pure** — no I/O, no state writes, no
  `console.log`. The compiler re-runs it at will.
- Any reactive state read inside the expression becomes a dependency.
- Dependencies are discovered per run; conditional reads work
  correctly.

### `$derived.by(() => ...)` — function form

When the expression needs multiple statements or early returns:

```ts
const tax = $derived.by(() => {
  if (country === 'US') return price * 0.07
  if (country === 'KR') return price * 0.10
  return 0
})
```

Same rules as `$derived` — purity is non-negotiable.

### Don't put effects in a derived

A `$derived` that logs, fetches, or mutates is a bug. Move that to
`$effect` or the triggering event handler.

## `$effect` — side effects

`$effect(fn)` runs `fn` after the component mounts, and again whenever
any reactive value it read changes.

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

Cleanup is the return value. Return `undefined` if there's nothing to
clean up.

### `$effect.pre` — before DOM commit

Rare, but needed for layout reads (measuring before paint):

```ts
$effect.pre(() => {
  const rect = el.getBoundingClientRect()
  // ...
})
```

Most effects should be plain `$effect`.

### `$effect.root` — imperative lifetimes

For effects owned outside a component (e.g. in a `.svelte.ts` module
that's initialized once):

```ts
const destroy = $effect.root(() => {
  $effect(() => { /* ... */ })
  return () => { /* root teardown */ }
})
```

Call `destroy()` when done. Required when runes run outside a
component boundary — the compiler otherwise can't tell when the
effect should stop.

### Golden rule

**`$effect` is for the outside world**. DOM APIs, timers,
subscriptions, analytics, logging. It is **not** a re-run hook for
your own state transitions.

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

- Always destructure. There's no "props object" to pass around.
- Defaults go on the destructure, not in the type.
- Typing is a regular TS destructure annotation — no special syntax.

### Renaming, rest, forwarding

```ts
// Rename: "class" is a JS keyword
let { class: className, ...rest } = $props()

// Forward remaining props to an inner element
<button class={className} {...rest}>{title}</button>
```

### Reading without destructuring

`$props()` returns a proxy-like object. If you genuinely need the full
bag without naming fields, you can hold it as a whole:

```ts
const all = $props()
console.log(all.title, all.count)
```

Rarely useful. Prefer destructuring so props are visible at a glance.

## `$bindable` — opt-in two-way binding

A plain prop is read-only. To let the parent two-way bind with
`bind:value`, opt in with `$bindable`:

```svelte
<!-- TextField.svelte -->
<script>
  let { value = $bindable('') } = $props()
</script>
<input bind:value />

<!-- parent -->
<TextField bind:value={formState.name} />
```

The default (first arg to `$bindable`) is used when the parent didn't
bind. Without `$bindable`, parent `bind:value={...}` on this
component is a compile error — which is good, because most props
shouldn't be two-way.

## `$inspect` — dev-only logging

```ts
$inspect(count, doubled)             // logs every time they change
$inspect('search:', query).with((type, value) => {
  if (type === 'update') sendMetric(value)
})
```

- Stripped from production builds.
- `.with` gives a custom handler (initial vs update events).
- Great for debugging reactivity — when you expect a re-run and it
  doesn't fire, `$inspect` tells you whether the value actually
  changed.

## `$host` — custom elements only

Inside `<svelte:options customElement="...">` components, `$host()`
returns the hosting custom element. Dispatch DOM events, read
attributes the browser set, etc. Not relevant for normal Svelte
apps.

## Decision tree

- "I have a value that changes" → `$state`
- "I have a value computed from others" → `$derived` (or
  `$derived.by`)
- "I need to do something when a value changes (outside Svelte)" →
  `$effect`
- "I'm writing a component" → `$props` (every time)
- "I want `bind:value` on my component" → `$bindable`
- "Why did / didn't this re-run?" → `$inspect`
