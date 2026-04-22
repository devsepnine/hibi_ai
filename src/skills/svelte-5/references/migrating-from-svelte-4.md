# Migrating from Svelte 4 to Svelte 5

Svelte 5 keeps `.svelte` files and most template syntax, and ships
with a **legacy mode** that recognizes v4 patterns for backwards
compatibility. New code should use runes mode (the default). This
doc covers the mechanical translations plus the subtle cases where a
blind swap goes wrong.

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
| `<slot {item} />` inside `{#each}` | pass an argumented snippet: `row: Snippet<[Item]>` |
| `<svelte:component this={Comp} />` | `<Comp />` (components are values) |
| `writable(0)` in `.ts` | `let v = $state(0)` in `.svelte.ts` |
| `$store` auto-subscribe | `store.value` (if using a runed store) |
| `bind:this={el}` | same — unchanged |
| `<svelte:self />` | same — unchanged |

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

Points of care:

- TypeScript types go in the destructure annotation, not via JSDoc.
- Defaults are normal JS destructure defaults.
- Props are **reactive by default** — reads track, no `$:` needed.

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

`$:` in v4 did two things that runes now split:

- **Computed values** → `$derived`
- **Side effects** → `$effect`

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

The third case (writing to a dep inside a `$:`) was subtly broken in
v4 and is explicitly a pattern-smell in v5 — if you port it
mechanically to `$effect`, you get an infinite loop. Move the
condition into whatever event triggered the update.

## Events

v4 used event directives (`on:click`) with modifier shortcuts. v5
uses plain attributes.

### v4

```svelte
<button on:click={handle} on:click|preventDefault={handle2} />
```

### v5

```svelte
<button onclick={handle} />
<button onclick={(e) => { e.preventDefault(); handle2(e) }} />
```

Modifiers (`|preventDefault`, `|stopPropagation`, `|once`,
`|capture`, `|self`, `|trusted`, `|nonpassive`, `|passive`) don't
exist in v5. Replace with ordinary JS:

| v4 modifier | v5 equivalent |
|-------------|---------------|
| `preventDefault` | call `e.preventDefault()` at the top |
| `stopPropagation` | call `e.stopPropagation()` |
| `once` | wrap handler to unsubscribe after first call |
| `capture` | add event listener manually with `addEventListener('click', h, true)` (or use an attachment) |
| `self` | check `e.target === e.currentTarget` |
| `trusted` | check `e.isTrusted` |
| `passive` / `nonpassive` | use `@attach` with an explicit `addEventListener(..., { passive: ... })` |

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

The child calls the callback directly — no `.detail`, no
`CustomEvent`. Types flow naturally through the prop.

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

Snippets are first-class values — you can pass multiple, accept
them as props, and type their arguments. More power than slots
(the old `let:` syntax is gone).

### Named slots

v4's `<slot name="header" />` → a named prop of type `Snippet`,
rendered with `{@render header?.()}`. The parent writes
`{#snippet header()}...{/snippet}` inside the component tag.

## Stores

The `svelte/store` API still works — but new code should prefer
runes in `.svelte.ts` modules.

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

Benefits:

- No subscribe/unsubscribe dance.
- Works in classes and objects (exposed via getters).
- TypeScript types flow without `Writable<T>` wrappers.

If you keep v4 stores (gradual migration), they still auto-subscribe
in `.svelte` files with the `$` prefix.

## Dynamic components

v4:
```svelte
<svelte:component this={Component} {...props} />
```

v5:
```svelte
<Component {...props} />
```

Components are ordinary values in v5 — assign them to variables,
pass them through props, pick one in a `$derived`. `<svelte:component>`
still compiles (legacy mode) but is unnecessary.

## Mixed mode

Svelte 5 can run v4-style files alongside runes-mode files.
Important rules:

- **Don't mix in one file.** A single component is either legacy or
  runes — using `$state` in a file that also has `export let` fails
  to compile.
- **Runes mode is opt-in per file** — writing any rune (`$state`,
  `$props`, etc.) flips the file. Otherwise the compiler keeps v4
  semantics.
- **You can explicitly set** `<svelte:options runes={true} />` or
  `<svelte:options runes={false} />` if the heuristic fails (rare).

Recommended migration path for a large app:

1. Upgrade to Svelte 5 with no code changes — legacy mode keeps it
   working.
2. Migrate leaf components first (fewest dependencies).
3. Move to `.svelte.ts` for shared state; delete the store versions.
4. Migrate layouts and routes last.
5. Once everything is runes, delete `"svelteOptions.legacy"` compiler
   flags you may have set.

## Automated migration

Svelte ships `sv migrate svelte-5`:

```bash
npx sv migrate svelte-5
```

This handles the mechanical translations (props, `$:`, `on:event`,
`<slot>`) file-by-file with prompts for ambiguous cases. It's safe
to run on a branch, review the diff, and pick what to keep.

It does **not** convert stores to `.svelte.ts` modules (too
project-specific) or restructure components that mix concerns —
save those for a human pass.

## Gotchas worth calling out

- **Legacy mode doesn't give you runes.** A v4 file opened in a v5
  project still uses `$:` reactivity, even though the rest of the app
  is running under a v5 compiler. Don't sprinkle `$state` into it
  hoping to "partially upgrade."
- **`$$props` / `$$restProps` are legacy-only.** In runes mode, use
  `let { ...rest } = $props()`.
- **`on:event` on DOM elements** still compiles under legacy mode —
  but in runes mode, switch to plain attributes. The types for
  `on:*` are less precise than the DOM attribute types.
- **`$app/stores`** works but logs a deprecation in SvelteKit 2.
  Migrate to `$app/state` (no `$` prefix, plain reactive access).
- **`beforeNavigate` / `afterNavigate` lifecycle hooks** moved to
  `$app/navigation` — if you were importing from `svelte`, stop.
