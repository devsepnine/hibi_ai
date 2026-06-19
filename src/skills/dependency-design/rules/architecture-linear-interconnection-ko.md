---
title: "Control Interconnection Complexity: Linear Flow + Message Constraints"
impact: MEDIUM
impactDescription: interconnection complexity ripples across modules and cannot be contained the way modular components can
tags: architecture, interconnection, pipeline
---

## Control Interconnection Complexity: Linear Flow + Message Constraints

시스템은 **components + interconnections + purpose**다. component는 모듈화되어 있으므로 변경의 blast radius가 그 내부로 한정된다. interconnection 복잡성은 다르다. 모듈 경계로 가둘 수 없기 때문에, 그 여파는 닿는 모든 것으로 번진다. 그래서 연결 설계에는 별도의 규율이 필요하다. encapsulation이 그것을 흡수해 주리라 기대할 수 없다.

세 가지 제약이 interconnection 복잡성을 통제한다.

- **연결을 linear하게 만들라.** linear interconnection은 시간적 개념이다. 연결이 정해진 순서(sequential / pipelined)로 일어난다. 순서는 인과관계를 담으므로, 전체 흐름을 머릿속에 담지 않고도 한 단계씩 추론할 수 있다. 이것이 부분 분석과 부분 수정을 가능하게 하는 pipelining 전략이다.
- **연결을 unidirectional하게 유지하라.** 연결은 request -> response 한 방향이지, 양방향으로 수다스럽게 주고받는 대화가 아니다. request를 보내려면 대상을 알아야 하므로, 방향은 곧 dependency 방향과 같으며 one-way 방향은 인과적 순서도 확정한다. 단일 간선만으로는 양방향이 아니어도 큰 순환을 통해 간접적으로 양방향이 되는 경우를 경계하라.
- **메시지를 제약하라.** 연결을 가로지르는 메시지의 schema를 좁혀, 잘못된 입력이 퍼지기 전에 경계에서 차단되게 하라. 느슨한 payload는 검증을 하류로 밀어내고, 그곳에서는 이미 여파가 넓어져 있다.

**Incorrect:**

```typescript
// Bidirectional, chatty, permissive — the caller and worker call back and forth,
// and the message is an open bag of optional fields validated nowhere.
interface Job {
  kind?: string
  payload?: unknown        // anything goes; errors surface deep downstream
  onProgress?: (pct: number) => void
}

class Worker {
  constructor(private caller: Caller) {}            // worker knows the caller
  run(job: Job) {
    this.caller.notifyStarted()                     // back-edge to caller
    const data = this.caller.fetchMore(job.kind)    // pulls more mid-run
    job.onProgress?.(50)                            // calls back in
    this.caller.notifyDone(data)                    // and again -> cycle
  }
}
```

**Correct:**

```typescript
// One-way pipeline stage with a strict, validated message schema.
import { z } from "zod"

const RenderRequest = z.object({
  documentId: z.string().uuid(),
  pages: z.array(z.number().int().positive()).nonempty(),
})
type RenderRequest = z.infer<typeof RenderRequest>

interface RenderResult {
  documentId: string
  url: string
}

// Each stage takes a validated input and RETURNS an output. No call-backs,
// no reference to the caller. The pipeline composes the stages in order.
function renderStage(input: unknown): RenderResult {
  const req = RenderRequest.parse(input)   // invalid message rejected at the edge
  const url = render(req.documentId, req.pages)
  return { documentId: req.documentId, url }
}

// request -> response, one way; the orchestrator owns the ordering.
const result = renderStage(incomingMessage)
```

Reference: [Complexity, Cynefin, and Degrees of Freedom](../references/complexity.md)
