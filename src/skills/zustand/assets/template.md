```typescript
import { fetchItems } from './api.js';
import { create } from 'zustand';
import { devtools, persist, createJSONStorage } from 'zustand/middleware';
import { useShallow } from 'zustand/shallow';
import type {} from '@redux-devtools/extension';

// ============================================================================
// Types
// ============================================================================

export interface {{StoreName}}Item {
  id: string;
}

function parseItem(value: unknown): {{StoreName}}Item {
  if (typeof value === 'object' && value !== null && 'id' in value) {
    const id = value.id;
    if (typeof id === 'string') return { id };
  }
  throw new TypeError('Expected an item with a string ID');
}

function parseItems(value: unknown): {{StoreName}}Item[] {
  if (!Array.isArray(value)) throw new TypeError('Expected an array of items');
  const items: {{StoreName}}Item[] = [];
  for (const item of value) {
    items.push(parseItem(item));
  }
  return items;
}

/**
 * {{StoreName}}State — data shape (no functions).
 */
export interface {{StoreName}}State {
  items: {{StoreName}}Item[];
  selectedId: string | null;
  load: { status: 'idle' } | { status: 'loading' } | { status: 'failed'; error: string };
}

/**
 * {{StoreName}}Actions — mutators and async operations.
 */
export interface {{StoreName}}Actions {
  // Setters
  setItems: (items: {{StoreName}}Item[]) => void;
  setSelectedId: (id: string | null) => void;

  // Complex / async
  loadItems: () => Promise<void>;
  addItem: (item: {{StoreName}}Item) => void;
  removeItem: (id: string) => void;

  // Reset to initialState (stays in sync with store.getInitialState())
  reset: () => void;
}

export type {{StoreName}}Store = {{StoreName}}State & {{StoreName}}Actions;

// ============================================================================
// Initial State
// ============================================================================

const initialState: {{StoreName}}State = {
  items: [],
  selectedId: null,
  load: { status: 'idle' },
};

// ============================================================================
// Store
// ============================================================================

/**
 * use{{StoreName}}Store — Zustand v5 store for managing {{description}}.
 *
 * Middleware chain:
 *   devtools → persist → state creator
 *
 * Selectors:
 *   - Single field:  const items = useStore((s) => s.items)
 *   - Multi-field:   const { items, add } = useStore(useShallow((s) => ({ ... })))
 *
 * Outside-React subscribe: consider adding `subscribeWithSelector` if needed.
 */
export const use{{StoreName}}Store = create<{{StoreName}}Store>()(
  devtools(
    persist(
      (set, get) => ({
        ...initialState,

        // Simple setters — name the action for devtools tracing
        setItems: (items) => set({ items: parseItems(items) }, false, '{{StoreName}}/setItems'),
        setSelectedId: (selectedId) =>
          set({ selectedId }, false, '{{StoreName}}/setSelectedId'),

        // Replacing load on retry or reset invalidates the previous request.
        loadItems: async () => {
          const request = { status: 'loading' } as const;
          set({ load: request }, false, '{{StoreName}}/loadItems/start');
          try {
            const payload: unknown = await fetchItems();
            if (get().load !== request) return;
            const items = parseItems(payload);
            set({ items, load: { status: 'idle' } }, false, '{{StoreName}}/loadItems/success');
          } catch (err) {
            if (get().load !== request) return;
            set(
              {
                load: { status: 'failed', error: err instanceof Error ? err.message : 'Failed to load' },
              },
              false,
              '{{StoreName}}/loadItems/error',
            );
          }
        },

        // Immutable updates via spread / filter
        addItem: (item) => {
          const storedItem = parseItem(item);
          set({ items: [...get().items, storedItem] }, false, '{{StoreName}}/addItem');
        },

        removeItem: (id) =>
          set(
            {
              items: get().items.filter((item) => item.id !== id),
              selectedId: get().selectedId === id ? null : get().selectedId,
            },
            false,
            '{{StoreName}}/removeItem',
          ),

        // Reset to initialState — also reachable via store.getInitialState()
        reset: () => set(initialState, false, '{{StoreName}}/reset'),
      }),
      {
        name: '{{storeName}}-storage',
        storage: createJSONStorage(() => localStorage),
        // Persist only user-facing selections; drop transient state
        partialize: (s) => ({ selectedId: s.selectedId }),
      },
    ),
    { name: '{{StoreName}}Store' },
  ),
);

// ============================================================================
// Selector helpers (optional — export hooks co-located with the store)
// ============================================================================

/** Multi-field selector example — use `useShallow` to avoid infinite loops. */
export const use{{StoreName}}List = () =>
  use{{StoreName}}Store(
    useShallow((s) => ({
      items: s.items,
      isLoading: s.load.status === 'loading',
      error: s.load.status === 'failed' ? s.load.error : null,
    })),
  );
```
