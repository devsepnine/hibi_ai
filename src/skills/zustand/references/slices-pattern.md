# Zustand Slices Pattern

Split a growing Zustand store into domain-aligned slices that compose into
one hook. Each slice is a `StateCreator` parameterized by the full store
type so cross-slice `get()` calls remain type-safe.

Use this pattern when:
- A single store exceeds ~5 actions or mixes unrelated domains
- Multiple teams/features contribute to the same global store
- You want tree-shakeable, independently testable slice modules

Skip it when the store is small (3-4 actions) — the extra generics hurt
readability more than they help.

## Minimal Example

```typescript
import { create, StateCreator } from 'zustand';

// ----- Slice types -----
interface BearSlice {
  bears: number;
  addBear: () => void;
  eatFish: () => void; // reaches into FishSlice via get()
}

interface FishSlice {
  fishes: number;
  addFish: () => void;
}

type JungleStore = BearSlice & FishSlice;

// ----- Slice creators -----
const createBearSlice: StateCreator<JungleStore, [], [], BearSlice> = (set) => ({
  bears: 0,
  addBear: () => set((s) => ({ bears: s.bears + 1 })),
  eatFish: () => set((s) => ({ fishes: s.fishes - 1 })),
});

const createFishSlice: StateCreator<JungleStore, [], [], FishSlice> = (set) => ({
  fishes: 0,
  addFish: () => set((s) => ({ fishes: s.fishes + 1 })),
});

// ----- Compose -----
export const useJungleStore = create<JungleStore>()((...a) => ({
  ...createBearSlice(...a),
  ...createFishSlice(...a),
}));
```

The `StateCreator<Full, Mutators, UnusedMutators, Slice>` generic pattern
lets each slice see the full store shape via `set`/`get`, while only
returning its own slice. The second/third generics are middleware mutator
metadata — use `[]` when no middleware applies at the slice level.

## Cross-slice dependencies

Keep cross-slice reads inside the slice that owns the action. Don't spread
`get()` calls across components.

```typescript
interface SharedSlice {
  addBoth: () => void;
  getBoth: () => number;
}

const createSharedSlice: StateCreator<
  BearSlice & FishSlice & SharedSlice,
  [],
  [],
  SharedSlice
> = (_, get) => ({
  addBoth: () => {
    get().addBear();
    get().addFish();
  },
  getBoth: () => get().bears + get().fishes,
});
```

## Composing with middleware (devtools)

When the final store wears middleware, each slice's `StateCreator` must
advertise the mutator so types line up. For `devtools`:

```typescript
import { create, StateCreator } from 'zustand';
import { devtools } from 'zustand/middleware';

type BearSlice = { bears: number; addBear: () => void };
type FishSlice = { fishes: number; addFish: () => void };
type JungleStore = BearSlice & FishSlice;

const createBearSlice: StateCreator<
  JungleStore,
  [['zustand/devtools', never]], //  <-- declares the mutator
  [],
  BearSlice
> = (set) => ({
  bears: 0,
  addBear: () =>
    set((s) => ({ bears: s.bears + 1 }), undefined, 'jungle/bear/addBear'),
});

const createFishSlice: StateCreator<
  JungleStore,
  [['zustand/devtools', never]],
  [],
  FishSlice
> = (set) => ({
  fishes: 0,
  addFish: () =>
    set((s) => ({ fishes: s.fishes + 1 }), undefined, 'jungle/fish/addFish'),
});

export const useJungleStore = create<JungleStore>()(
  devtools((...a) => ({
    ...createBearSlice(...a),
    ...createFishSlice(...a),
  })),
);
```

Rule: **add mutator declarations at the slice level for any middleware that
wraps the composed store.** Common ones:
- `['zustand/devtools', never]`
- `['zustand/persist', unknown]`
- `['zustand/immer', never]`

## Organizing slice files

```
store/
├── index.ts              # export useJungleStore + typed selectors
├── jungle-store.ts       # create() + compose
├── slices/
│   ├── bear-slice.ts     # BearSlice + createBearSlice
│   ├── fish-slice.ts
│   └── shared-slice.ts
└── __tests__/
    └── jungle-store.test.ts
```

Each slice file exports its type and creator only — no `create()` call.
This keeps tree-shaking clean and lets tests exercise slices in isolation.

## Testing a sliced store

```typescript
import { useJungleStore } from './jungle-store';

beforeEach(() => {
  useJungleStore.setState(useJungleStore.getInitialState(), true);
});

test('addBoth increments both counters', () => {
  useJungleStore.getState().addBoth();
  const { bears, fishes } = useJungleStore.getState();
  expect(bears).toBe(1);
  expect(fishes).toBe(1);
});
```

For application-wide reset (e.g., logout), use the global-reset wrapper
from the v5 docs — register each store's `setState(initial, true)` with a
shared reset registry.

## Common mistakes

- **Typing slice creators with only the slice shape** — `get()` loses cross-slice
  visibility. Always parameterize with the *full* store type.
- **Forgetting the mutator tuple** when composing under `devtools`/`persist` —
  TypeScript errors on `set(partial, false, 'name')` because the third
  parameter only exists with the mutator declared.
- **Splitting too early** — if the store has 3 actions, inline it. Slices earn
  their cost around 5+ actions or across domain boundaries.
- **Cross-slice state duplication** — don't mirror another slice's field;
  read it via `get()` in the action that needs it.
