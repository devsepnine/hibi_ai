# React Composition Patterns

**Version 1.0.0**  
Engineering  
January 2026

> **Note:**  
> 이 문서는 주로 에이전트와 LLM이 composition을 사용하는 React 코드베이스를  
> 유지보수, 생성, 리팩토링할 때 따르도록 작성되었다. 사람도 유용하게 활용할 수  
> 있지만, 여기의 가이드는 AI 보조 워크플로우의 자동화와 일관성에 최적화되어  
> 있다.

---

## Abstract

유연하고 유지보수 가능한 React 컴포넌트를 구축하기 위한 composition 패턴이다. compound component 사용, state lifting, 내부 구성요소 composition을 통해 boolean prop 증식을 피한다. 이러한 패턴은 코드베이스가 확장될 때 사람과 AI 에이전트 모두가 작업하기 쉽도록 만든다.

---

## Table of Contents

1. [Component Architecture](#1-component-architecture) — **HIGH**
   - 1.1 [Avoid Boolean Prop Proliferation](#11-avoid-boolean-prop-proliferation)
   - 1.2 [Use Compound Components](#12-use-compound-components)
2. [State Management](#2-state-management) — **MEDIUM**
   - 2.1 [Decouple State Management from UI](#21-decouple-state-management-from-ui)
   - 2.2 [Define Generic Context Interfaces for Dependency Injection](#22-define-generic-context-interfaces-for-dependency-injection)
   - 2.3 [Lift State into Provider Components](#23-lift-state-into-provider-components)
3. [Implementation Patterns](#3-implementation-patterns) — **MEDIUM**
   - 3.1 [Create Explicit Component Variants](#31-create-explicit-component-variants)
   - 3.2 [Prefer Composing Children Over Render Props](#32-prefer-composing-children-over-render-props)
4. [React 19 APIs](#4-react-19-apis) — **MEDIUM**
   - 4.1 [React 19 API Changes](#41-react-19-api-changes)

---

## 1. Component Architecture

**Impact: HIGH**

prop 증식을 피하고 유연한 composition을 가능하게 하는 컴포넌트 구조화의 기본 패턴이다.

### 1.1 Avoid Boolean Prop Proliferation

**Impact: CRITICAL (prevents unmaintainable component variants)**

`isThread`, `isEditing`, `isDMThread` 같은 boolean prop을 추가해 컴포넌트 동작을 커스터마이즈하지 않는다. 각 boolean은 가능한 상태를 두 배로 늘리고 유지보수가 불가능한 조건부 로직을 만든다. 대신 composition을 사용한다.

**Incorrect: boolean props create exponential complexity**

```tsx
function Composer({
  onSubmit,
  isThread,
  channelId,
  isDMThread,
  dmId,
  isEditing,
  isForwarding,
}: Props) {
  return (
    <form>
      <Header />
      <Input />
      {isDMThread ? (
        <AlsoSendToDMField id={dmId} />
      ) : isThread ? (
        <AlsoSendToChannelField id={channelId} />
      ) : null}
      {isEditing ? (
        <EditActions />
      ) : isForwarding ? (
        <ForwardActions />
      ) : (
        <DefaultActions />
      )}
      <Footer onSubmit={onSubmit} />
    </form>
  )
}
```

**Correct: composition eliminates conditionals**

```tsx
// Channel composer
function ChannelComposer() {
  return (
    <Composer.Frame>
      <Composer.Header />
      <Composer.Input />
      <Composer.Footer>
        <Composer.Attachments />
        <Composer.Formatting />
        <Composer.Emojis />
        <Composer.Submit />
      </Composer.Footer>
    </Composer.Frame>
  )
}

// Thread composer - adds "also send to channel" field
function ThreadComposer({ channelId }: { channelId: string }) {
  return (
    <Composer.Frame>
      <Composer.Header />
      <Composer.Input />
      <AlsoSendToChannelField id={channelId} />
      <Composer.Footer>
        <Composer.Formatting />
        <Composer.Emojis />
        <Composer.Submit />
      </Composer.Footer>
    </Composer.Frame>
  )
}

// Edit composer - different footer actions
function EditComposer() {
  return (
    <Composer.Frame>
      <Composer.Input />
      <Composer.Footer>
        <Composer.Formatting />
        <Composer.Emojis />
        <Composer.CancelEdit />
        <Composer.SaveEdit />
      </Composer.Footer>
    </Composer.Frame>
  )
}
```

각 변형은 무엇을 렌더링하는지 명시적이다. 단일한 거대한 부모를 공유하지 않고도 내부 구성요소를 공유할 수 있다.

### 1.2 Use Compound Components

**Impact: HIGH (enables flexible composition without prop drilling)**

복잡한 컴포넌트를 공유 context를 가진 compound component로 구조화한다. 각 하위 컴포넌트는 prop이 아닌 context를 통해 공유 상태에 접근한다. 소비자는 필요한 부분만 조합한다.

**Incorrect: monolithic component with render props**

```tsx
function Composer({
  renderHeader,
  renderFooter,
  renderActions,
  showAttachments,
  showFormatting,
  showEmojis,
}: Props) {
  return (
    <form>
      {renderHeader?.()}
      <Input />
      {showAttachments && <Attachments />}
      {renderFooter ? (
        renderFooter()
      ) : (
        <Footer>
          {showFormatting && <Formatting />}
          {showEmojis && <Emojis />}
          {renderActions?.()}
        </Footer>
      )}
    </form>
  )
}
```

**Correct: compound components with shared context**

```tsx
const ComposerContext = createContext<ComposerContextValue | null>(null)

function ComposerProvider({ children, state, actions, meta }: ProviderProps) {
  return (
    <ComposerContext value={{ state, actions, meta }}>
      {children}
    </ComposerContext>
  )
}

function ComposerFrame({ children }: { children: React.ReactNode }) {
  return <form>{children}</form>
}

function ComposerInput() {
  const {
    state,
    actions: { update },
    meta: { inputRef },
  } = use(ComposerContext)
  return (
    <TextInput
      ref={inputRef}
      value={state.input}
      onChangeText={(text) => update((s) => ({ ...s, input: text }))}
    />
  )
}

function ComposerSubmit() {
  const {
    actions: { submit },
  } = use(ComposerContext)
  return <Button onPress={submit}>Send</Button>
}

// Export as compound component
const Composer = {
  Provider: ComposerProvider,
  Frame: ComposerFrame,
  Input: ComposerInput,
  Submit: ComposerSubmit,
  Header: ComposerHeader,
  Footer: ComposerFooter,
  Attachments: ComposerAttachments,
  Formatting: ComposerFormatting,
  Emojis: ComposerEmojis,
}
```

**Usage:**

```tsx
<Composer.Provider state={state} actions={actions} meta={meta}>
  <Composer.Frame>
    <Composer.Header />
    <Composer.Input />
    <Composer.Footer>
      <Composer.Formatting />
      <Composer.Submit />
    </Composer.Footer>
  </Composer.Frame>
</Composer.Provider>
```

소비자는 필요한 것을 정확히 명시적으로 조합한다. 숨겨진 조건문이 없다. 그리고 state, actions, meta는 부모 provider에 의해 의존성 주입되어 동일한 컴포넌트 구조를 여러 용도로 사용할 수 있게 한다.

---

## 2. State Management

**Impact: MEDIUM**

조합된 컴포넌트 전반에 걸쳐 상태를 끌어올리고 공유 context를 관리하는 패턴이다.

### 2.1 Decouple State Management from UI

**Impact: MEDIUM (enables swapping state implementations without changing UI)**

provider 컴포넌트는 상태가 어떻게 관리되는지를 아는 유일한 곳이어야 한다. UI 컴포넌트는 context 인터페이스를 소비할 뿐 — 상태가 useState, Zustand, 서버 동기화 중 어디서 오는지 알지 못한다.

**Incorrect: UI coupled to state implementation**

```tsx
function ChannelComposer({ channelId }: { channelId: string }) {
  // UI component knows about global state implementation
  const state = useGlobalChannelState(channelId)
  const { submit, updateInput } = useChannelSync(channelId)

  return (
    <Composer.Frame>
      <Composer.Input
        value={state.input}
        onChange={(text) => sync.updateInput(text)}
      />
      <Composer.Submit onPress={() => sync.submit()} />
    </Composer.Frame>
  )
}
```

**Correct: state management isolated in provider**

```tsx
// Provider handles all state management details
function ChannelProvider({
  channelId,
  children,
}: {
  channelId: string
  children: React.ReactNode
}) {
  const { state, update, submit } = useGlobalChannel(channelId)
  const inputRef = useRef(null)

  return (
    <Composer.Provider
      state={state}
      actions={{ update, submit }}
      meta={{ inputRef }}
    >
      {children}
    </Composer.Provider>
  )
}

// UI component only knows about the context interface
function ChannelComposer() {
  return (
    <Composer.Frame>
      <Composer.Header />
      <Composer.Input />
      <Composer.Footer>
        <Composer.Submit />
      </Composer.Footer>
    </Composer.Frame>
  )
}

// Usage
function Channel({ channelId }: { channelId: string }) {
  return (
    <ChannelProvider channelId={channelId}>
      <ChannelComposer />
    </ChannelProvider>
  )
}
```

**Different providers, same UI:**

```tsx
// Local state for ephemeral forms
function ForwardMessageProvider({ children }) {
  const [state, setState] = useState(initialState)
  const forwardMessage = useForwardMessage()

  return (
    <Composer.Provider
      state={state}
      actions={{ update: setState, submit: forwardMessage }}
    >
      {children}
    </Composer.Provider>
  )
}

// Global synced state for channels
function ChannelProvider({ channelId, children }) {
  const { state, update, submit } = useGlobalChannel(channelId)

  return (
    <Composer.Provider state={state} actions={{ update, submit }}>
      {children}
    </Composer.Provider>
  )
}
```

동일한 `Composer.Input` 컴포넌트가 두 provider 모두에서 작동한다. 구현이 아닌 context 인터페이스에만 의존하기 때문이다.

### 2.2 Define Generic Context Interfaces for Dependency Injection

**Impact: HIGH (enables dependency-injectable state across use-cases)**

컴포넌트 context를 위한 **제네릭 인터페이스**를 세 부분으로 정의한다: `state`, `actions`, `meta`. 이 인터페이스는 어떤 provider든 구현할 수 있는 계약이며 — 같은 UI 컴포넌트가 완전히 다른 상태 구현과 함께 작동하도록 한다.

**Core principle:** 상태를 끌어올리고, 내부 구성요소를 조합하며, 상태를 의존성 주입 가능하게 만든다.

**Incorrect: UI coupled to specific state implementation**

```tsx
function ComposerInput() {
  // Tightly coupled to a specific hook
  const { input, setInput } = useChannelComposerState()
  return <TextInput value={input} onChangeText={setInput} />
}
```

**Correct: generic interface enables dependency injection**

```tsx
// Define a GENERIC interface that any provider can implement
interface ComposerState {
  input: string
  attachments: Attachment[]
  isSubmitting: boolean
}

interface ComposerActions {
  update: (updater: (state: ComposerState) => ComposerState) => void
  submit: () => void
}

interface ComposerMeta {
  inputRef: React.RefObject<TextInput>
}

interface ComposerContextValue {
  state: ComposerState
  actions: ComposerActions
  meta: ComposerMeta
}

const ComposerContext = createContext<ComposerContextValue | null>(null)
```

**UI components consume the interface, not the implementation:**

```tsx
function ComposerInput() {
  const {
    state,
    actions: { update },
    meta,
  } = use(ComposerContext)

  // This component works with ANY provider that implements the interface
  return (
    <TextInput
      ref={meta.inputRef}
      value={state.input}
      onChangeText={(text) => update((s) => ({ ...s, input: text }))}
    />
  )
}
```

**Different providers implement the same interface:**

```tsx
// Provider A: Local state for ephemeral forms
function ForwardMessageProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState(initialState)
  const inputRef = useRef(null)
  const submit = useForwardMessage()

  return (
    <ComposerContext
      value={{
        state,
        actions: { update: setState, submit },
        meta: { inputRef },
      }}
    >
      {children}
    </ComposerContext>
  )
}

// Provider B: Global synced state for channels
function ChannelProvider({ channelId, children }: Props) {
  const { state, update, submit } = useGlobalChannel(channelId)
  const inputRef = useRef(null)

  return (
    <ComposerContext
      value={{
        state,
        actions: { update, submit },
        meta: { inputRef },
      }}
    >
      {children}
    </ComposerContext>
  )
}
```

**The same composed UI works with both:**

```tsx
// Works with ForwardMessageProvider (local state)
<ForwardMessageProvider>
  <Composer.Frame>
    <Composer.Input />
    <Composer.Submit />
  </Composer.Frame>
</ForwardMessageProvider>

// Works with ChannelProvider (global synced state)
<ChannelProvider channelId="abc">
  <Composer.Frame>
    <Composer.Input />
    <Composer.Submit />
  </Composer.Frame>
</ChannelProvider>
```

**Custom UI outside the component can access state and actions:**

```tsx
function ForwardMessageDialog() {
  return (
    <ForwardMessageProvider>
      <Dialog>
        {/* The composer UI */}
        <Composer.Frame>
          <Composer.Input placeholder="Add a message, if you'd like." />
          <Composer.Footer>
            <Composer.Formatting />
            <Composer.Emojis />
          </Composer.Footer>
        </Composer.Frame>

        {/* Custom UI OUTSIDE the composer, but INSIDE the provider */}
        <MessagePreview />

        {/* Actions at the bottom of the dialog */}
        <DialogActions>
          <CancelButton />
          <ForwardButton />
        </DialogActions>
      </Dialog>
    </ForwardMessageProvider>
  )
}

// This button lives OUTSIDE Composer.Frame but can still submit based on its context!
function ForwardButton() {
  const {
    actions: { submit },
  } = use(ComposerContext)
  return <Button onPress={submit}>Forward</Button>
}

// This preview lives OUTSIDE Composer.Frame but can read composer's state!
function MessagePreview() {
  const { state } = use(ComposerContext)
  return <Preview message={state.input} attachments={state.attachments} />
}
```

중요한 것은 provider 경계지 시각적 중첩이 아니다. 공유 상태가 필요한 컴포넌트는 `Composer.Frame` 내부에 있을 필요가 없다. provider 내부에 있기만 하면 된다.

`ForwardButton`과 `MessagePreview`는 시각적으로 composer 박스 안에 있지 않지만, 여전히 그 상태와 actions에 접근할 수 있다. 이것이 상태를 provider로 끌어올리는 것의 힘이다.

UI는 함께 조합하는 재사용 가능한 조각이다. 상태는 provider에 의해 의존성 주입된다. provider를 바꾸고 UI는 유지한다.

### 2.3 Lift State into Provider Components

**Impact: HIGH (enables state sharing outside component boundaries)**

상태 관리를 전용 provider 컴포넌트로 옮긴다. 이를 통해 메인 UI 외부의 형제 컴포넌트가 prop drilling이나 어색한 ref 없이 상태에 접근하고 수정할 수 있다.

**Incorrect: state trapped inside component**

```tsx
function ForwardMessageComposer() {
  const [state, setState] = useState(initialState)
  const forwardMessage = useForwardMessage()

  return (
    <Composer.Frame>
      <Composer.Input />
      <Composer.Footer />
    </Composer.Frame>
  )
}

// Problem: How does this button access composer state?
function ForwardMessageDialog() {
  return (
    <Dialog>
      <ForwardMessageComposer />
      <MessagePreview /> {/* Needs composer state */}
      <DialogActions>
        <CancelButton />
        <ForwardButton /> {/* Needs to call submit */}
      </DialogActions>
    </Dialog>
  )
}
```

**Incorrect: useEffect to sync state up**

```tsx
function ForwardMessageDialog() {
  const [input, setInput] = useState('')
  return (
    <Dialog>
      <ForwardMessageComposer onInputChange={setInput} />
      <MessagePreview input={input} />
    </Dialog>
  )
}

function ForwardMessageComposer({ onInputChange }) {
  const [state, setState] = useState(initialState)
  useEffect(() => {
    onInputChange(state.input) // Sync on every change 😬
  }, [state.input])
}
```

**Incorrect: reading state from ref on submit**

```tsx
function ForwardMessageDialog() {
  const stateRef = useRef(null)
  return (
    <Dialog>
      <ForwardMessageComposer stateRef={stateRef} />
      <ForwardButton onPress={() => submit(stateRef.current)} />
    </Dialog>
  )
}
```

**Correct: state lifted to provider**

```tsx
function ForwardMessageProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState(initialState)
  const forwardMessage = useForwardMessage()
  const inputRef = useRef(null)

  return (
    <Composer.Provider
      state={state}
      actions={{ update: setState, submit: forwardMessage }}
      meta={{ inputRef }}
    >
      {children}
    </Composer.Provider>
  )
}

function ForwardMessageDialog() {
  return (
    <ForwardMessageProvider>
      <Dialog>
        <ForwardMessageComposer />
        <MessagePreview /> {/* Custom components can access state and actions */}
        <DialogActions>
          <CancelButton />
          <ForwardButton /> {/* Custom components can access state and actions */}
        </DialogActions>
      </Dialog>
    </ForwardMessageProvider>
  )
}

function ForwardButton() {
  const { actions } = use(Composer.Context)
  return <Button onPress={actions.submit}>Forward</Button>
}
```

ForwardButton은 Composer.Frame 외부에 있지만 여전히 submit 액션에 접근할 수 있다. provider 내부에 있기 때문이다. 일회성 컴포넌트라 하더라도 UI 자체 외부에서 composer의 상태와 액션에 접근할 수 있다.

**Key insight:** 공유 상태가 필요한 컴포넌트는 서로 시각적으로 중첩될 필요가 없다 — 같은 provider 내에 있기만 하면 된다.

---

## 3. Implementation Patterns

**Impact: MEDIUM**

compound component와 context provider 구현을 위한 구체적인 기법이다.

### 3.1 Create Explicit Component Variants

**Impact: MEDIUM (self-documenting code, no hidden conditionals)**

여러 boolean prop을 가진 하나의 컴포넌트 대신, 명시적인 변형 컴포넌트를 만든다. 각 변형은 필요한 부분을 조합한다. 코드가 스스로 문서화된다.

**Incorrect: one component, many modes**

```tsx
// What does this component actually render?
<Composer
  isThread
  isEditing={false}
  channelId='abc'
  showAttachments
  showFormatting={false}
/>
```

**Correct: explicit variants**

```tsx
// Immediately clear what this renders
<ThreadComposer channelId="abc" />

// Or
<EditMessageComposer messageId="xyz" />

// Or
<ForwardMessageComposer messageId="123" />
```

각 구현은 고유하고 명시적이며 자기 완결적이다. 그러면서도 공유 부품을 사용할 수 있다.

**Implementation:**

```tsx
function ThreadComposer({ channelId }: { channelId: string }) {
  return (
    <ThreadProvider channelId={channelId}>
      <Composer.Frame>
        <Composer.Input />
        <AlsoSendToChannelField channelId={channelId} />
        <Composer.Footer>
          <Composer.Formatting />
          <Composer.Emojis />
          <Composer.Submit />
        </Composer.Footer>
      </Composer.Frame>
    </ThreadProvider>
  )
}

function EditMessageComposer({ messageId }: { messageId: string }) {
  return (
    <EditMessageProvider messageId={messageId}>
      <Composer.Frame>
        <Composer.Input />
        <Composer.Footer>
          <Composer.Formatting />
          <Composer.Emojis />
          <Composer.CancelEdit />
          <Composer.SaveEdit />
        </Composer.Footer>
      </Composer.Frame>
    </EditMessageProvider>
  )
}

function ForwardMessageComposer({ messageId }: { messageId: string }) {
  return (
    <ForwardMessageProvider messageId={messageId}>
      <Composer.Frame>
        <Composer.Input placeholder="Add a message, if you'd like." />
        <Composer.Footer>
          <Composer.Formatting />
          <Composer.Emojis />
          <Composer.Mentions />
        </Composer.Footer>
      </Composer.Frame>
    </ForwardMessageProvider>
  )
}
```

각 변형은 다음에 대해 명시적이다:

- 어떤 provider/state를 사용하는가

- 어떤 UI 요소를 포함하는가

- 어떤 액션을 사용할 수 있는가

따져볼 boolean prop 조합이 없다. 불가능한 상태가 없다.

### 3.2 Prefer Composing Children Over Render Props

**Impact: MEDIUM (cleaner composition, better readability)**

`renderX` prop 대신 `children`을 composition에 사용한다. children은 더 읽기 쉽고, 자연스럽게 조합되며, 콜백 시그니처를 이해할 필요가 없다.

**Incorrect: render props**

```tsx
function Composer({
  renderHeader,
  renderFooter,
  renderActions,
}: {
  renderHeader?: () => React.ReactNode
  renderFooter?: () => React.ReactNode
  renderActions?: () => React.ReactNode
}) {
  return (
    <form>
      {renderHeader?.()}
      <Input />
      {renderFooter ? renderFooter() : <DefaultFooter />}
      {renderActions?.()}
    </form>
  )
}

// Usage is awkward and inflexible
return (
  <Composer
    renderHeader={() => <CustomHeader />}
    renderFooter={() => (
      <>
        <Formatting />
        <Emojis />
      </>
    )}
    renderActions={() => <SubmitButton />}
  />
)
```

**Correct: compound components with children**

```tsx
function ComposerFrame({ children }: { children: React.ReactNode }) {
  return <form>{children}</form>
}

function ComposerFooter({ children }: { children: React.ReactNode }) {
  return <footer className='flex'>{children}</footer>
}

// Usage is flexible
return (
  <Composer.Frame>
    <CustomHeader />
    <Composer.Input />
    <Composer.Footer>
      <Composer.Formatting />
      <Composer.Emojis />
      <SubmitButton />
    </Composer.Footer>
  </Composer.Frame>
)
```

**When render props are appropriate:**

```tsx
// Render props work well when you need to pass data back
<List
  data={items}
  renderItem={({ item, index }) => <Item item={item} index={index} />}
/>
```

부모가 자식에게 데이터나 상태를 제공해야 할 때 render props를 사용한다. 정적인 구조를 조합할 때는 children을 사용한다.

---

## 4. React 19 APIs

**Impact: MEDIUM**

React 19+ only. `forwardRef`를 사용하지 않는다; `useContext()` 대신 `use()`를 사용한다.

### 4.1 React 19 API Changes

**Impact: MEDIUM (cleaner component definitions and context usage)**

> **⚠️ React 19+ only.** React 18 이하라면 이 절은 건너뛴다.

React 19에서는 `ref`가 일반 prop이 되었으며 (`forwardRef` 래퍼 불필요), `use()`가 `useContext()`를 대체한다.

**Incorrect: forwardRef in React 19**

```tsx
const ComposerInput = forwardRef<TextInput, Props>((props, ref) => {
  return <TextInput ref={ref} {...props} />
})
```

**Correct: ref as a regular prop**

```tsx
function ComposerInput({ ref, ...props }: Props & { ref?: React.Ref<TextInput> }) {
  return <TextInput ref={ref} {...props} />
}
```

**Incorrect: useContext in React 19**

```tsx
const value = useContext(MyContext)
```

**Correct: use instead of useContext**

```tsx
const value = use(MyContext)
```

`use()`는 `useContext()`와 달리 조건부 호출도 가능하다.

---

## References

1. [https://react.dev](https://react.dev)
2. [https://react.dev/learn/passing-data-deeply-with-context](https://react.dev/learn/passing-data-deeply-with-context)
3. [https://react.dev/reference/react/use](https://react.dev/reference/react/use)
