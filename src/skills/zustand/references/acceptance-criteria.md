# Zustand Store Acceptance Criteria (TypeScript, v5)

**Library**: Zustand v5+
**Purpose**: Validate generated Zustand code against current (v5) best
practices. Includes a Migration section for changes from v4.

---

## 1. Import Patterns

### 1.1 ✅ CORRECT

```typescript
import { create } from 'zustand';
import { useShallow } from 'zustand/shallow';
import {
  devtools,
  persist,
  createJSONStorage,
  subscribeWithSelector,
} from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import type {} from '@redux-devtools/extension'; // for devtools typing
```

### 1.2 ❌ INCORRECT

```typescript
// Default import removed in v4+
import create from 'zustand';

// Old path — useShallow moved to 'zustand/shallow' in v5
import { useShallow } from 'zustand/react/shallow';

// `shallow` as selector equality arg removed from v5 `create`
// (use `useShallow` hook, or `createWithEqualityFn` from `zustand/traditional`)
import { shallow } from 'zustand/shallow';
```

---

## 2. Store Creation

### 2.1 ✅ CORRECT: Plain store (no middleware needed)

```typescript
import { create } from 'zustand';

interface MyStore {
  count: number;
  increment: () => void;
}

export const useMyStore = create<MyStore>()((set) => ({
  count: 0,
  increment: () => set((s) => ({ count: s.count + 1 })),
}));
```

`subscribeWithSelector` is **not required**. Add it only when the store is
subscribed to from outside React (see §5).

### 2.2 ✅ CORRECT: With `get()`

```typescript
export const useMyStore = create<MyStore>()((set, get) => ({
  count: 0,
  increment: () => set((s) => ({ count: s.count + 1 })),
  double: () => set({ count: get().count * 2 }),
}));
```

### 2.3 ✅ CORRECT: Generic syntax with middleware

Note the **double parentheses** — `create<T>()(middleware(...))`. The empty
pair enables type inference through middleware.

```typescript
export const useMyStore = create<MyStore>()(
  devtools((set) => ({ /* ... */ })),
);
```

### 2.4 ❌ INCORRECT: Missing `()` on generic with middleware

```typescript
// Generic inference breaks — always use create<T>()(...) shape
export const useMyStore = create<MyStore>(
  devtools((set) => ({ /* ... */ })),
);
```

---

## 3. State and Actions Separation

### 3.1 ✅ CORRECT

```typescript
export interface ProjectState {
  projects: Project[];
  selectedId: string | null;
  isLoading: boolean;
}

export interface ProjectActions {
  addProject: (project: Project) => void;
  selectProject: (id: string) => void;
  loadProjects: () => Promise<void>;
  reset: () => void;
}

export type ProjectStore = ProjectState & ProjectActions;

const initialState: ProjectState = {
  projects: [],
  selectedId: null,
  isLoading: false,
};

export const useProjectStore = create<ProjectStore>()((set, get) => ({
  ...initialState,
  addProject: (project) =>
    set((s) => ({ projects: [...s.projects, project] })),
  selectProject: (selectedId) => set({ selectedId }),
  loadProjects: async () => {
    set({ isLoading: true });
    try {
      const projects = await fetchProjects();
      set({ projects, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },
  reset: () => set(initialState),
}));
```

---

## 4. Selector Patterns

### 4.1 ✅ CORRECT: Single-field selectors

Individual selectors are the safest form — no wrapper needed.

```typescript
const count = useMyStore((s) => s.count);
const increment = useMyStore((s) => s.increment);
```

### 4.2 ✅ CORRECT: `useShallow` for arrays, tuples, objects

v5 uses reference equality, so a selector returning a *new array/object each
render* causes an infinite loop. Wrap with `useShallow`.

```typescript
import { useShallow } from 'zustand/shallow';

// Object literal — wrap
const { count, isLoading } = useMyStore(
  useShallow((s) => ({ count: s.count, isLoading: s.isLoading })),
);

// Tuple/array — wrap
const [count, increment] = useMyStore(
  useShallow((s) => [s.count, s.increment]),
);
```

### 4.3 ❌ INCORRECT: Unwrapped multi-field selector

```typescript
// Causes infinite re-render in v5
const { count, isLoading } = useMyStore((s) => ({
  count: s.count,
  isLoading: s.isLoading,
}));
```

### 4.4 ❌ INCORRECT: Destructuring the whole store

```typescript
// Re-renders on any state change
const { count, isLoading } = useMyStore();
```

### 4.5 Alternative: `createWithEqualityFn`

If an existing codebase relies on `create(..., shallow)` equality, migrate
the specific store to `createWithEqualityFn` from `zustand/traditional`
rather than refactoring every call site.

```typescript
import { createWithEqualityFn as create } from 'zustand/traditional';
import { shallow } from 'zustand/shallow';

const useStore = createWithEqualityFn<MyStore>()((set) => ({ /* ... */ }));

// v4-style selector still works
const { count, text } = useStore((s) => ({ count: s.count, text: s.text }), shallow);
```

New code should prefer `useShallow`.

---

## 5. Subscribe Outside React (`subscribeWithSelector`)

Include `subscribeWithSelector` only when subscribing outside React
(event bridges, logging, URL sync, analytics). Skip it otherwise.

### 5.1 ✅ CORRECT

```typescript
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

export const useMyStore = create<MyStore>()(
  subscribeWithSelector((set) => ({ /* ... */ })),
);

const unsubscribe = useMyStore.subscribe(
  (s) => s.selectedId,
  (selectedId, prev) => console.log('selected:', prev, '->', selectedId),
  { fireImmediately: true, equalityFn: Object.is },
);
```

---

## 6. Persist Middleware

### 6.1 ✅ CORRECT: With partialize

```typescript
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set) => ({
      theme: 'dark',
      tempToken: null, // transient, don't persist
      setTheme: (theme) => set({ theme }),
    }),
    {
      name: 'settings-storage',
      storage: createJSONStorage(() => localStorage),
      partialize: (s) => ({ theme: s.theme }), // persist only `theme`
    },
  ),
);
```

### 6.2 Versioning & migration

```typescript
persist(
  (set) => ({ /* ... */ }),
  {
    name: 'settings-storage',
    version: 2,
    migrate: (persisted: any, fromVersion) => {
      if (fromVersion < 2) return { ...persisted, theme: persisted.color };
      return persisted;
    },
  },
);
```

---

## 7. Devtools Middleware

### 7.1 ✅ CORRECT

```typescript
import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import type {} from '@redux-devtools/extension';

export const useMyStore = create<MyStore>()(
  devtools(
    (set) => ({
      count: 0,
      increment: () =>
        set((s) => ({ count: s.count + 1 }), false, 'MyStore/increment'),
    }),
    { name: 'MyStore' },
  ),
);
```

Name every `set()` call — the third argument shows up in Redux DevTools.

### 7.2 Compose order (common chain)

```typescript
// devtools wraps persist wraps the state creator
create<MyStore>()(devtools(persist((set) => ({ /* ... */ }), { name: '...' })));
```

---

## 8. Immer Middleware

### 8.1 ✅ CORRECT

```typescript
import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';

export const useTodoStore = create<TodoStore>()(
  immer((set) => ({
    todos: [],
    addTodo: (todo) =>
      set((s) => {
        s.todos.push(todo); // mutation ok inside immer
      }),
    toggleTodo: (id) =>
      set((s) => {
        const todo = s.todos.find((t) => t.id === id);
        if (todo) todo.completed = !todo.completed;
      }),
  })),
);
```

---

## 9. Async Actions

### 9.1 ✅ CORRECT

```typescript
export const useDataStore = create<DataStore>()((set) => ({
  data: null,
  isLoading: false,
  error: null,

  fetchData: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      const data = await api.getData(id);
      set({ data, isLoading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Request failed',
        isLoading: false,
      });
    }
  },
}));
```

Always reset `error` on start, surface a readable message, and clear
loading on every exit path.

---

## 10. Slices Pattern

Use `StateCreator<Full, Mutators, UnusedMutators, Slice>` for modular stores.
See [slices-pattern.md](./slices-pattern.md) for the full guide.

### 10.1 ✅ CORRECT

```typescript
import { create, StateCreator } from 'zustand';

interface UISlice {
  isSidebarOpen: boolean;
  toggleSidebar: () => void;
}

interface DataSlice {
  items: Item[];
  addItem: (item: Item) => void;
}

type AppStore = UISlice & DataSlice;

const createUISlice: StateCreator<AppStore, [], [], UISlice> = (set) => ({
  isSidebarOpen: true,
  toggleSidebar: () => set((s) => ({ isSidebarOpen: !s.isSidebarOpen })),
});

const createDataSlice: StateCreator<AppStore, [], [], DataSlice> = (set) => ({
  items: [],
  addItem: (item) => set((s) => ({ items: [...s.items, item] })),
});

export const useAppStore = create<AppStore>()((...a) => ({
  ...createUISlice(...a),
  ...createDataSlice(...a),
}));
```

### 10.2 ❌ INCORRECT: Old `SetState<T>` signature

```typescript
// Outdated — use StateCreator<Full, [], [], Slice> instead
const createUISlice = (set: SetState<AppStore>): UISlice => ({ /* ... */ });
```

---

## 11. Testing Patterns

### 11.1 ✅ CORRECT: Reset between tests via `getInitialState()`

```typescript
beforeEach(() => {
  // v5 API — no need to duplicate initialState manually
  useMyStore.setState(useMyStore.getInitialState(), true);
});

test('increment advances count', () => {
  useMyStore.getState().increment();
  expect(useMyStore.getState().count).toBe(1);
});
```

### 11.2 ✅ CORRECT: Global reset registry (e.g., on logout)

```typescript
import { create as actualCreate, type StateCreator } from 'zustand';

const storeResetFns = new Set<() => void>();

export const resetAllStores = () => {
  storeResetFns.forEach((fn) => fn());
};

export const create = (<T>() => (creator: StateCreator<T>) => {
  const store = actualCreate<T>()(creator);
  storeResetFns.add(() => store.setState(store.getInitialState(), true));
  return store;
}) as typeof actualCreate;
```

---

## 12. Migration: v4 → v5

| v4 | v5 | Notes |
|----|----|-------|
| `create(..., shallow)` in selector | `useShallow((s) => ...)` hook | Preferred |
| `create(..., shallow)` in selector | `createWithEqualityFn` + `shallow` | Legacy path, use `zustand/traditional` |
| `from 'zustand/react/shallow'` | `from 'zustand/shallow'` | Path renamed |
| Manual `initialState` capture | `store.getInitialState()` | New v5 API |
| Any object/array selector without wrapper | Wrap with `useShallow` | v5 no longer memoizes by default |

The key behavioral change: v5 matches React's default reference equality, so
any selector returning a fresh reference must be wrapped.

---

## 13. Anti-Patterns Summary

```typescript
// ❌ Default import (pre-v4)
import create from 'zustand';

// ❌ Whole-store destructure — re-renders on every state change
const { count, items } = useStore();

// ❌ Unwrapped multi-field selector — infinite loop in v5
const data = useStore((s) => ({ count: s.count }));

// ❌ Unwrapped tuple selector — infinite loop in v5
const [a, b] = useStore((s) => [s.a, s.b]);

// ❌ Old useShallow path
import { useShallow } from 'zustand/react/shallow';

// ❌ Mutating without immer
set((s) => { s.items.push(item); return s; });

// ❌ Old SetState<T> slice signature
const createSlice = (set: SetState<Store>): Slice => ({ /* ... */ });

// ❌ Missing double parentheses with middleware
create<Store>(devtools(/* ... */)); // should be create<Store>()(devtools(/* ... */))
```
