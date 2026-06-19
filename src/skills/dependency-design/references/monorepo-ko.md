# Layered and Turbo Monorepo Architecture

## 모듈화는 일관된 기준을 가진 추상화다

시스템을 모듈로 나누는 일은 그 분할 뒤에 단 하나의 일관된 기준이 있을 때만
의미가 있으며, 그 기준은 곧 추상화다. 모듈은 하나의 일관된 지식 덩어리를 더
작은 표면 뒤에 감출 때 비로소 경계를 가질 자격을 얻는다. 추상화의 *수준* 이나
*종류* 가 모듈마다 들쭉날쭉하면 경계는 아무 의미도 갖지 못한다. 읽는 사람은
무엇이 어디에 있는지 더 이상 예측할 수 없고, 그 "모듈화"는 장식일 뿐이다.

따라서 어떤 경계든 긋기 전에 그 경계가 나타내는 추상화에 이름을 붙이고, 형제
모듈들이 같은 수준에 있는지 확인하라. "PDF 도메인"과 "날짜 포매팅 헬퍼"를
동등한 peer로 묶고 있다면 기준이 흐트러졌다는 신호다.

## 레이어 아키텍처: viewpoint에 따른 분리

레이어란 단 하나의 *viewpoint* — 시스템 전체에 균일하게 적용된, 선택된 하나의
추상화 축 — 를 따라 그어진 모듈 경계다. 모든 레이어가 그 축을 공유하기 때문에
레이어링은 언제나 **functional coupling** 을 만들어낸다. 각 레이어는 그 위
레이어를 위해 한 범주의 기능을 수행하려고 존재한다.

레이어링에는 알아보기 쉬운 trade-off 프로파일이 있다.

- 전통적인 조직이 일을 나누는 방식과 닮아 있어, 잘 이해된(확정된) 도메인에서
  자연스럽게 읽힌다.
- 단방향 의존성을 표현하기 쉽다 — 상위 레이어가 아래로만 의존하고, 그 반대는
  없다.
- 그러나 **N:N coupling 이 자주 발생한다**: 상위 레이어의 여러 호출자가 하위
  레이어의 여러 provider를 건드리므로 엣지가 늘어난다.
- 자원 소비 확장에 유연하게 대응하며, 각 관심사가 정확히 한 레이어에만 살기
  때문에 문제가 발생한 지점을 *특정* 하기 좋다.

N:N 경향은 순수한 수평 분할의 대가다. 모든 엣지가 여전히 한 방향을 가리키는 한
받아들일 만하며, 분할 축이 일관되지 않을 때만(위의 모듈화 기준 참고) 문제가
된다.

## 레이어링에 쓸 수 있는 세 가지 viewpoint

같은 시스템을 둘 이상의 viewpoint로 자를 수 있다. 다음 세 가지가 유용하며,
진짜 힘은 이들을 합성하는 데서 나온다.

### 1. 라이프사이클 viewpoint

*인스턴스가 얼마나 오래 살고 어떤 상태를 갖는가* 로 레이어를 나눈다.

- **Presentation** — 상황이 필요할 때마다 생성되고, 처리되면 파기된다. 가장
  짧게 산다.
- **Application** — 한 번의 request→response 동안 살고, 처리 후 파기된다.
- **Business logic** — 상대적으로 long-term으로 유지된다. 상태 없는 불변식
  (요청과 무관하게 성립하는 규칙).
- **Data access** — 상대적으로 long-term으로 유지된다. 영구 상태 관리만
  처리한다.

### 2. 기능 역할 viewpoint

*일을 끝내는 데 각 부분이 어떤 역할을 하는가* 로 나눈다.

- **Interface** (presentation) — 최초 이벤트를 발생시키고 결과를 수령한다.
- **Orchestrator** (application) — 기능을 모아 중계/순서 배열하며, 자신만의
  실제 작업은 갖지 않는다.
- **Providers** (business logic + data access) — 실제로 기능을 제공하는
  레이어.

### 3. 도메인 역할 viewpoint

*각 부분이 도메인을 얼마나 인지하는가* 로 나눈다.

- **Domain layer** — 도메인별 상호작용을 담당한다.
- **Function layer** — 도메인이 활용할 수 있는 도메인 중립적 기능.
- **Foundation layer** — function layer가 동작할 수 있게 하는 기반 기능.

이 viewpoint들은 서로 직교한다. 하나의 물리적 단위가 각 viewpoint에서 서로 다른
위치를 차지할 수 있으며, 아래의 monorepo 레이아웃이 바로 이 점을 활용한다.

## Turbo monorepo: 복합 레이어링

Turbo monorepo는 세 viewpoint를 모두 하나의 물리적 구조 — `apps/*` 와
`packages/*` — 로 합친다. 각 최상위 위치는 각 viewpoint에서 알려진 위치로
매핑된다.

```
apps/<app>          orchestrator + 독립 배포 가능한 application
packages/<domain>   business-logic + data-access (도메인의 불변식과 long-term
                    상태)
packages/<lib>      순수 foundation 기능
```

### `apps/*` — orchestrator이자 배포 단위

`app`은 **orchestrator** 다. `packages`가 공개한 기능을 직접 구현하는 대신
조합한다. 또한 라이프사이클/배포 의미에서의 **application** 이기도 하다 — 독립적
으로 빌드/배포 가능한 단위(front-end, server 등).

엄격한 규칙: `app → packages`는 **언제나** 단방향이다. app은 package에 의존할 수
있지만, package는 결코 app에 의존해서는 안 된다.

### `packages/<domain>*` — business logic + data access

도메인 package는 도메인의 **불변식**(상태 없는 비즈니스 규칙)과 **long-term 상태
관리**(data access)를 담는다. 실제 도메인 작업이 사는 곳이다.

### `packages/<lib>*` — 순수 foundation

lib package는 **foundation layer** 다. business-logic과 data-access 코드가
의존하는 중립적이고 재사용 가능한 기능(HTTP fetch, socket, LLM API client 등).

### package 사이의 의존성 규칙

```
apps/*  ──────────────▶  packages/*          (항상 허용, 단방향)
packages/<domain>  ───▶  packages/<lib>       (허용, 단방향)
packages/<lib>     ──╳   packages/<lib>       (지양)
```

- `app → packages`: 항상 단방향, 그 반대는 없다.
- `packages/<domain> → packages/<lib>`: 단방향. 도메인이 foundation에 기댄다.
- `packages/<lib> → packages/<lib>`: **지양한다.** lib가 정말로 다른 lib를
  필요로 한다면, 그 공유 의존성은 `node_modules`에 속한다 — 즉, repo 안의 한
  foundation package가 옆으로 다른 foundation package에 손을 뻗게 두는 대신
  외부 의존성으로 publish하라. 이렇게 하면 foundation layer가 평평하게 유지되고
  내부 coupling의 숨은 두 번째 레이어가 생기는 것을 막는다.

전반의 목표는 모든 엣지가 한 방향을 가리키게 하고, 변동성이 크고 도메인을 아는
코드가 안정적인 foundation 코드에 끌려 들어오지 않게 하는 것이다.

## 도메인 package 세분화

하나의 도메인 package 안에서는 각 규칙을 소유하는 시스템 부분으로 나눈다.

```
packages/<domain>/
  common/    protocol, type, rule, invariant   (front와 server가 공유)
  front/     front-side domain rule, invariant
  server/    server-side domain rule, invariant
```

- `common`은 계약이다. protocol, 공유 type, 그리고 양쪽이 모두 지켜야 하는
  invariant.
- `front`와 `server`는 각각 해당 runtime에 특화된 규칙과 invariant를 `common`
  위에 얹는다.

이렇게 하면 front-end app과 server app이 각각 필요한 slice만 import하면서도,
공유 protocol/type은 정확히 한 곳에 머문다.

## 구체적인 트리

두 도메인(`ndx`, `pdf`)을 각각 front app과 server app으로 출하하고, 도메인
package와 foundation lib 묶음을 함께 둔 예다.

```
apps/
  ndxFront    presentation + domain + orchestrator
  ndxServer   application + orchestrator
  pdfFront
  pdfServer

packages/
  ndx/
    common    protocol, type, rule, invariant
    front     front domain rule, invariant
    server    server domain rule, invariant
  pdf/
    common
    front
    server
  pdfUtil     foundation
  llmApi      foundation
  socket      foundation
  fetch       foundation
```

viewpoint에 비추어 트리를 읽어 보면: `ndxFront`는 동시에 presentation
interface이자 domain 참여자이자 orchestrator이고, `ndxServer`는 application
라이프사이클의 orchestrator다. 둘 다 도메인 불변식과 상태를 `packages/ndx`에서
가져오며, 그 package는 다시 `fetch`나 `socket` 같은 foundation lib에 기댄다.
어떤 package도 app을 거꾸로 가리키지 않고, 어떤 foundation lib도 다른 lib를
옆으로 가리키지 않는다.

## 코드에서는 이렇게 나타난다

위의 판단은 이 skill에서 구체적인 규칙으로 강제된다.

- **Dependency Direction & Structure** — 이 레이아웃이 인코딩하는 단방향,
  비순환, change-rate 격리 엣지(`app → packages → lib`).
- **Abstraction & Module Boundary** — 모듈 경계를 그을 가치가 있게 만드는
  일관된 추상화 기준.
- **Layered & Monorepo Architecture** — `apps/*` 대 `packages/*` 배치 규칙과
  `common`/`front`/`server` 도메인 분할.

도메인 package 내부의 repository/data-access 경계는 `backend-patterns` skill을
참고하라.
