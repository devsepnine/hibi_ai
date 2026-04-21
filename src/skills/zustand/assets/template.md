```typescript
import { create } from 'zustand';
import { devtools, persist, createJSONStorage } from 'zustand/middleware';
import { useShallow } from 'zustand/shallow';
import type {} from '@redux-devtools/extension';

// ============================================================================
// Types
// ============================================================================

/**
 * {{StoreName}}State — data shape (no functions).
 */
export interface {{StoreName}}State {
  items: unknown[];
  selectedId: string | null;
  isLoading: boolean;
  error: string | null;
}

/**
 * {{StoreName}}Actions — mutators and async operations.
 */
export interface {{StoreName}}Actions {
  // Setters
  setItems: (items: unknown[]) => void;
  setSelectedId: (id: string | null) => void;

  // Complex / async
  loadItems: () => Promise<void>;
  addItem: (item: unknown) => void;
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
  isLoading: false,
  error: null,
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
        setItems: (items) => set({ items }, false, '{{StoreName}}/setItems'),
        setSelectedId: (selectedId) =>
          set({ selectedId }, false, '{{StoreName}}/setSelectedId'),

        // Async example — always clear error on start, set on failure
        loadItems: async () => {
          set({ isLoading: true, error: null }, false, '{{StoreName}}/loadItems/start');
          try {
            // const items = await fetchItems();
            const items: unknown[] = [];
            set({ items, isLoading: false }, false, '{{StoreName}}/loadItems/success');
          } catch (err) {
            set(
              {
                error: err instanceof Error ? err.message : 'Failed to load',
                isLoading: false,
              },
              false,
              '{{StoreName}}/loadItems/error',
            );
          }
        },

        // Immutable updates via spread / filter
        addItem: (item) =>
          set({ items: [...get().items, item] }, false, '{{StoreName}}/addItem'),

        removeItem: (id) =>
          set(
            {
              items: get().items.filter((it) => (it as { id: string }).id !== id),
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
      isLoading: s.isLoading,
      error: s.error,
    })),
  );
```
