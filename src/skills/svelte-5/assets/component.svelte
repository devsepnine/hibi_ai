<script lang="ts">
  // -----------------------------------------------------------------------
  // 1) Imports — types first, then Svelte APIs, then local helpers
  // -----------------------------------------------------------------------
  import type { Snippet } from 'svelte'
  import { slide } from 'svelte/transition'

  // -----------------------------------------------------------------------
  // 2) Props — always $props, destructured, typed
  // -----------------------------------------------------------------------
  type Props = {
    /** Accordion heading text */
    title: string
    /** Whether the panel starts open */
    open?: boolean
    /** Called whenever the user toggles the panel */
    ontoggle?: (open: boolean) => void
    /** Body content; rendered via {@render children?.()} */
    children?: Snippet
  }

  let {
    title,
    open = false,
    ontoggle,
    children,
  }: Props = $props()

  // -----------------------------------------------------------------------
  // 3) Local reactive state
  // -----------------------------------------------------------------------
  let expanded = $state(open)

  // -----------------------------------------------------------------------
  // 4) Derived values — pure, no side effects
  // -----------------------------------------------------------------------
  const caret = $derived(expanded ? '▾' : '▸')
  const ariaLabel = $derived(expanded ? `Collapse ${title}` : `Expand ${title}`)

  // -----------------------------------------------------------------------
  // 5) Side effects — only for DOM / subscriptions / external calls,
  //    never for writing reactive state (that's what event handlers are for)
  // -----------------------------------------------------------------------
  $effect(() => {
    // Example: notify parent on every change, or persist to localStorage.
    ontoggle?.(expanded)
  })

  // -----------------------------------------------------------------------
  // 6) Event handlers
  // -----------------------------------------------------------------------
  function toggle() {
    expanded = !expanded
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      toggle()
    }
  }
</script>

<div class="accordion">
  <button
    class="heading"
    onclick={toggle}
    onkeydown={onKeyDown}
    aria-expanded={expanded}
    aria-label={ariaLabel}
  >
    <span class="caret" aria-hidden="true">{caret}</span>
    {title}
  </button>

  {#if expanded}
    <div class="panel" transition:slide={{ duration: 180 }}>
      {@render children?.()}
    </div>
  {/if}
</div>

<style>
  .accordion {
    border: 1px solid var(--border, #e5e7eb);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .heading {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: var(--heading-bg, #f9fafb);
    border: 0;
    font-weight: 600;
    cursor: pointer;
  }

  .heading:hover {
    background: var(--heading-bg-hover, #f3f4f6);
  }

  .caret {
    display: inline-block;
    width: 1em;
    text-align: center;
  }

  .panel {
    padding: 0.75rem 1rem;
  }
</style>
