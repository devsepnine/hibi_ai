---
name: code-http-test-generator
description: "IntelliJ .http와 .rest 파일로 API 계약 테스트와 HTTP 요청 예제를 작성한다."
---

# HTTP Client 테스트

1. 라우트와 관련 설정, OpenAPI 문서, 기존 테스트에서 요청 대상의 API 계약을 확인한다. HTTP 메서드, 접두 경로를 포함한 실제 경로, 입력, 콘텐츠 타입, 인증, 기대 상태 코드와 응답 필드를 확인한다. 근거가 서로 충돌하면 불확실한 동작을 단언에 넣기 전에 차이를 밝힌다.
2. 주변 `.http` 또는 `.rest` 파일의 관례를 따른다. 관련 없는 요청 블록과 기존 환경 이름을 유지한다. 각 요청에 내용을 드러내는 이름을 붙이고 앞선 요청의 데이터가 필요한 경우 실행 순서를 적는다. 계약으로 확인한 성공 사례와 발생 가능한 실패 사례를 포함한다. 단언은 사례별 계약을 따르고, 계약에서 명시한 경우에만 자동 생성 식별자나 타임스탬프에 의존한다.
3. 호스트, 픽스처 식별자, 자격 증명은 변수로 참조한다. 프로젝트에서 `http-client.env.json`을 쓰면 로컬 `baseUrl`처럼 공유 가능한 설정만 넣는다. 로컬 비밀값은 `http-client.private.env.json`이나 프로젝트의 기존 비밀값 저장소에서 가져온다. 실제 비밀값과 응답의 자격 증명이 추적 파일, 출력, 보고서에 포함되지 않게 한다. Git에서 비공개 환경 파일을 쓸 때는 `git check-ignore -v --no-index -- <path>`와 `git ls-files -- <path>`로 제외 규칙에 해당하고 추적되지 않는지 확인한다.

아래는 API 계약상 상태 코드 `200`을 반환하는 기존 픽스처를 조회하는 문법 예시다.

```http
### Read an existing item
GET {{baseUrl}}/items/{{itemId}}
Accept: application/json

> {%
  client.test("Returns the item", function () {
    client.assert(response.status === 200, "Expected an existing item");
  });
%}
```

요청 구분자, 변수 정의, 본문 경계, 응답 핸들러 위치를 검토한다. 실행 요청을 받으면 설정된 IntelliJ HTTP Client나 호환 실행기로 지정된 환경에서 실행한다.

요청이나 응답에 비밀값이 포함될 수 있는 테스트는 민감하지 않은 자격 증명과 픽스처 데이터를 사용한다. 실제 비밀값이 필요한 경우에는 실행 전에 요청 기록, 응답 파일, 로그, 기타 산출물에 비밀값이 남지 않도록 실행기가 설정됐는지 확인한다. IntelliJ에서 `@no-log`로 요청 기록을 끌 수 있다. 응답은 `.idea/httpRequests/` 아래에 별도 파일로 저장될 수 있다. 쿠키 저장소가 필요 없는 테스트에서는 `@no-cookie-jar`로 응답 쿠키가 저장되지 않게 한다. [JetBrains HTTP Client 가이드](https://www.jetbrains.com/help/idea/http-client-in-product-code-editor.html)와 [요청 지시문](https://www.jetbrains.com/help/idea/exploring-http-syntax.html)을 참고한다.

실행한 요청을 명시하고 실행 결과와 정적 검토 결과를 구분한다. 문법은 [JetBrains 문법 레퍼런스](https://www.jetbrains.com/help/idea/http-client-reference.html)를 참고한다.
