```typescript
import { create } from 'zustand';
import { devtools, persist, createJSONStorage } from 'zustand/middleware';
import { useShallow } from 'zustand/shallow';
import type {} from '@redux-devtools/extension';

// ============================================================================
// Types
// ============================================================================

/**
 * {{StoreName}}State — 데이터 형태 (함수 없음).
 */
export interface {{StoreName}}State {
  items: unknown[];
  selectedId: string | null;
  isLoading: boolean;
  error: string | null;
}

/**
 * {{StoreName}}Actions — 변경자(mutator)와 비동기 작업.
 */
export interface {{StoreName}}Actions {
  // 세터
  setItems: (items: unknown[]) => void;
  setSelectedId: (id: string | null) => void;

  // 복합 / 비동기
  loadItems: () => Promise<void>;
  addItem: (item: unknown) => void;
  removeItem: (id: string) => void;

  // initialState로 리셋 (store.getInitialState()와 동기 유지)
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
 * use{{StoreName}}Store — {{description}}을 관리하는 Zustand v5 스토어.
 *
 * 미들웨어 체인:
 *   devtools → persist → state creator
 *
 * 셀렉터:
 *   - 단일 필드:  const items = useStore((s) => s.items)
 *   - 다중 필드:  const { items, add } = useStore(useShallow((s) => ({ ... })))
 *
 * React 외부 구독: 필요하면 `subscribeWithSelector` 추가를 검토한다.
 */
export const use{{StoreName}}Store = create<{{StoreName}}Store>()(
  devtools(
    persist(
      (set, get) => ({
        ...initialState,

        // 단순 세터 — devtools 추적을 위해 액션 이름을 붙인다
        setItems: (items) => set({ items }, false, '{{StoreName}}/setItems'),
        setSelectedId: (selectedId) =>
          set({ selectedId }, false, '{{StoreName}}/setSelectedId'),

        // 비동기 예제 — 시작 시 항상 error를 비우고, 실패 시 설정한다
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

        // spread / filter를 통한 불변 업데이트
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

        // initialState로 리셋 — store.getInitialState()로도 도달 가능
        reset: () => set(initialState, false, '{{StoreName}}/reset'),
      }),
      {
        name: '{{storeName}}-storage',
        storage: createJSONStorage(() => localStorage),
        // 사용자에게 보이는 선택만 영구화하고 일시 상태는 버린다
        partialize: (s) => ({ selectedId: s.selectedId }),
      },
    ),
    { name: '{{StoreName}}Store' },
  ),
);

// ============================================================================
// Selector helpers (optional — 스토어와 함께 위치하는 훅을 export)
// ============================================================================

/** 다중 필드 셀렉터 예제 — 무한 루프를 피하려면 `useShallow`를 사용한다. */
export const use{{StoreName}}List = () =>
  use{{StoreName}}Store(
    useShallow((s) => ({
      items: s.items,
      isLoading: s.isLoading,
      error: s.error,
    })),
  );
```
