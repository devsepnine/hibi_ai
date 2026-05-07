# Zustand Slices Pattern

성장하는 Zustand store를 도메인별로 정렬된 slice로 분할하여 하나의 hook으로 조합한다. 각 slice는 전체 store 타입으로 매개변수화된 `StateCreator`이므로 cross-slice `get()` 호출이 type-safe로 유지된다.

이 패턴을 사용할 때:
- 단일 store가 ~5개 action을 초과하거나 관련 없는 도메인을 섞음
- 여러 팀/feature가 같은 글로벌 store에 기여
- tree-shakeable, 독립적으로 testable한 slice 모듈을 원함

store가 작을 때 (action 3-4개) 건너뛴다 — 추가 generic이 가독성에 도움보다 해가 된다.

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

`StateCreator<Full, Mutators, UnusedMutators, Slice>` generic 패턴은 각 slice가 `set`/`get`을 통해 전체 store 형태를 볼 수 있게 하면서 자체 slice만 반환하게 한다. 두 번째/세 번째 generic은 middleware mutator 메타데이터이다 — slice 레벨에 middleware가 적용되지 않을 때 `[]`을 사용한다.

## Cross-slice dependencies

cross-slice read를 action을 소유한 slice 내부에 유지한다. 컴포넌트 전반에 `get()` 호출을 흩뜨리지 마라.

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

최종 store가 middleware를 입을 때, 각 slice의 `StateCreator`는 타입이 일치하도록 mutator를 advertise해야 한다. `devtools`의 경우:

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

규칙: **조합된 store를 wrap하는 모든 middleware에 대해 slice 레벨에 mutator 선언을 추가한다.** 일반적인 것:
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

각 slice 파일은 자체 타입과 creator만 export한다 — `create()` 호출 없음. 이는 tree-shaking을 깨끗하게 유지하고 테스트가 slice를 격리해서 운동시킬 수 있게 한다.

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

애플리케이션 전체 reset (예: 로그아웃)은 v5 문서의 global-reset wrapper를 사용한다 — 각 store의 `setState(initial, true)`를 공유 reset registry에 등록한다.

## Common mistakes

- **slice 형태만으로 slice creator 타입 부여** — `get()`이 cross-slice 가시성을 잃는다. 항상 *전체* store 타입으로 매개변수화한다.
- **`devtools`/`persist` 하에 조합할 때 mutator tuple 잊기** — mutator가 선언되어야만 세 번째 매개변수가 존재하므로 TypeScript가 `set(partial, false, 'name')`에서 에러를 낸다.
- **너무 일찍 분할** — store에 action이 3개라면 inline한다. slice는 5개 이상의 action 또는 도메인 경계 전반에서 비용을 정당화한다.
- **cross-slice state 중복** — 다른 slice의 field를 mirror하지 마라; 그것을 필요로 하는 action에서 `get()`으로 read한다.
