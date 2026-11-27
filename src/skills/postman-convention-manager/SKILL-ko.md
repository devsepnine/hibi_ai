---
name: postman-convention-manager
description: "Postman 내보내기 파일의 변수 범위를 점검하고 이름 변경을 미리 보여준다."
---

# Postman 변수 규칙 점검

사용자가 지정한 로컬 내보내기 파일을 읽는다. 제안하는 이름과 영향을 받는 참조를 보고서에서 미리 보여준다. 입력 파일과 Postman 워크스페이스는 읽기 전용으로 다룬다.

## 점검 범위 확인

1. 제공된 컬렉션이나 환경 내보내기 파일을 JSON으로 파싱한다. 컬렉션 파일이 있으면 형식을 확인하고 v2.1에 해당할 때 아래 구조를 적용한다. 잘못된 JSON이나 지원하지 않는 형식은 파일 경로와 필요한 입력을 알려준다.
2. 사용자나 저장소가 정한 이름 규칙을 적용한다. 정해진 규칙이 없으면 내보내기 파일에서 관찰한 패턴을 설명하고 새 규칙은 제안으로 표시한다.
3. 제공된 컬렉션과 환경 내보내기 파일 범위를 기록하고, 확인할 수 있으면 활성 환경을 명시한다. 실행에 전역 변수, 반복 데이터, 로컬 변수가 쓰이면 제공된 정보를 이용하고 빠진 범위를 표시한다.

## 정의와 참조 추적

1. 컬렉션의 `variable`과 환경의 `values`에서 정의를 읽는다. 참조를 해석할 때 환경 변수가 켜져 있는지 확인한다. 중첩 폴더와 요청을 순회하고 컬렉션·폴더·요청 스크립트를 확인한다. 발견 사항마다 파일과 JSON 경로를 기록하고 위치를 찾는 데 도움이 되면 요청이나 폴더 이름을 덧붙인다.
2. 요청 URL, 파라미터, 헤더, 본문의 `{{key}}` 참조를 추적한다. 구조화된 URL 필드와 직렬화된 `raw` URL을 함께 확인한다. `{{$guid}}` 같은 내장 동적 변수는 이름 변경 제안에서 제외한다. 스크립트를 읽어 `pm.environment`, `pm.collectionVariables`, `pm.globals`, `pm.iterationData`, `pm.variables` 등 Postman 변수 API에 전달하는 리터럴 키를 살핀다. 정적 검사로 판별하기 어려운 계산된 키와 스크립트가 생성하는 참조를 표시한다.
   `/orders/:id` 같은 경로의 `:id`는 해당 요청의 `request.url.variable` 정의와 연결한다. 대응하는 정의가 없으면 미해결로 표시한다. 경로 변수 이름을 바꿀 때는 정의와 함께 `url.path` 및 `url.raw`에서 이름이 들어 있는 위치를 표시한다. 경로 변수 값에 들어 있는 `{{key}}`는 활성 범위에서 별도로 해석한다.
3. 컬렉션, 폴더, 요청의 `auth`에 있는 참조를 추적한다. `auth`가 없거나 `null`이면 인증 설정이 있는 가장 가까운 상위 항목을 찾는다. `noauth`를 포함해 명시한 설정이 상속보다 우선한다. 내보내기 파일에서 참조가 나온 위치와 해당 인증을 사용하는 요청을 기록한다.
4. 템플릿 참조와 `pm.variables.get()`는 활성 컨텍스트의 좁은 범위부터 확인한다. 넓은 범위부터 전역, 컬렉션, 환경, 데이터, 로컬 순서다. 상속한 인증의 참조는 실제 사용하는 요청의 컨텍스트에서 해석한다. 범위를 지정한 API는 해당 범위에서 해석한다. 제안한 이름이 기존 정의와 충돌하거나 참조가 가리키는 정의를 바꾸는지 점검한다.

## 미리보기 보고

- 검토한 내보내기 파일과 확인한 활성 범위를 명시한다.
- 변경 제안별로 기존 이름, 제안 이름, 정의 범위, 영향을 받는 참조 위치를 적는다. 이름 변경 제안이 없는 규칙 위반에도 위치를 붙인다.
- 변경에 영향을 주는 이름 충돌, 우선 참조 정의 변경, 알 수 없는 활성 범위, 동적 참조를 표시한다. 해결 전에는 해당 이름 변경을 구체적인 변경 미리보기에서 제외한다.
- 파일과 필드 위치를 알려주고 변수 값, 인증 정보, 요청 본문은 출력에서 제외한다.

[Postman 내보내기](https://learning.postman.com/docs/getting-started/importing-and-exporting/exporting-data) · [변수 범위](https://learning.postman.com/docs/use/send-requests/variables/variables/) · [환경 변수](https://learning.postman.com/docs/use/send-requests/variables/environment-variables) · [요청과 스크립트의 변수](https://learning.postman.com/latest-v-12/docs/use/send-requests/variables/use-variables) · [인증 상속](https://learning.postman.com/docs/use/send-requests/authorization/specifying-authorization-details#inherit-authorization)
