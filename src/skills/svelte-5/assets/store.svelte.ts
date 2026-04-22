// Example shared reactive state module using Svelte 5 runes.
// Filename MUST end in `.svelte.ts` (or `.svelte.js`) for runes to work
// outside a component.
//
// Usage:
//   import { cart } from './cart.svelte'
//   cart.add({ id: 'abc', name: 'Coffee', qty: 1 })
//   console.log(cart.total)  // reactive

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type CartItem = {
  id: string
  name: string
  qty: number
  unitPrice: number
}

// ---------------------------------------------------------------------------
// Reactive state — private to the module, exposed via getters
// ---------------------------------------------------------------------------

// Use $state for values that mutate (add/remove/increment).
let items = $state<CartItem[]>([])

// Use $state.raw for large, replace-only structures (not needed here, shown
// as a pattern when relevant):
// let snapshotHistory = $state.raw<CartItem[][]>([])

// $derived values are pure computations that re-run on dependency change.
const count = $derived(items.reduce((n, it) => n + it.qty, 0))
const total = $derived(
  items.reduce((n, it) => n + it.qty * it.unitPrice, 0),
)
const isEmpty = $derived(items.length === 0)

// ---------------------------------------------------------------------------
// Public API — object with getters + action methods
//
// Getters preserve reactivity at the consumption site: reading `cart.items`
// in a component registers a dependency on `items`, so the UI updates when
// the module mutates it.
// ---------------------------------------------------------------------------

export const cart = {
  get items() {
    return items
  },
  get count() {
    return count
  },
  get total() {
    return total
  },
  get isEmpty() {
    return isEmpty
  },

  /** Add an item; merges qty if the same id is already in the cart. */
  add(next: CartItem): void {
    const existing = items.find((it) => it.id === next.id)
    if (existing) {
      existing.qty += next.qty
      return
    }
    items.push(next)
  },

  /** Set qty absolutely; removes the item if qty <= 0. */
  setQty(id: string, qty: number): void {
    const existing = items.find((it) => it.id === id)
    if (!existing) return
    if (qty <= 0) {
      items = items.filter((it) => it.id !== id)
      return
    }
    existing.qty = qty
  },

  /** Remove a single line by id. Safe to call when id is unknown. */
  remove(id: string): void {
    items = items.filter((it) => it.id !== id)
  },

  /** Wipe the cart — usually on successful checkout. */
  clear(): void {
    items = []
  },
}

// ---------------------------------------------------------------------------
// Optional: alternative class-based shape when you want multiple carts
// (e.g., one per tab, or scoped to a user). Runes work in class fields too.
//
// export class Cart {
//   items = $state<CartItem[]>([])
//   total = $derived(this.items.reduce((n, it) => n + it.qty * it.unitPrice, 0))
//
//   add(it: CartItem) { this.items.push(it) }
//   clear()           { this.items = [] }
// }
//
// Then: const mainCart = new Cart();
// ---------------------------------------------------------------------------
