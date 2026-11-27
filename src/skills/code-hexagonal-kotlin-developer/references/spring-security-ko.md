# MVC와 WebFlux의 Spring Security

해당 라우트를 실제로 제공하는 HTTP 스택과 기존 보안 설정을 확인한다. Servlet 기반 MVC는 `HttpSecurity`와 `SecurityFilterChain`을, WebFlux는 `ServerHttpSecurity`와 `SecurityWebFilterChain`을 사용한다. 체인이 여러 개라면 매처와 순서를 확인한다. 변경한 라우트가 각각 어느 체인으로 들어가는지, 어느 체인에도 매칭되지 않는 라우트가 어떻게 처리되는지 확인한다. 공식 문서는 [Servlet 설정](https://docs.spring.io/spring-security/reference/servlet/configuration/java.html)과 [WebFlux 설정](https://docs.spring.io/spring-security/reference/reactive/configuration/webflux.html)을 참고한다.

접근 규칙은 API 계약을 따른다. CSRF 설정을 변경하기 전에 쿠키와 브라우저가 자동으로 첨부하는 기타 자격 증명을 포함해 인증 정보가 전달되는 방식을 확인한다. 브라우저의 교차 출처 요청을 받는 앱에서는 적용되는 CORS 설정을 확인하고 인증 전에 프리플라이트가 처리되도록 한다. 클라이언트에 실제로 응답하는 경계에서 응답 헤더를 점검한다. 프록시가 헤더를 추가하거나 덮어쓰면 프록시도 확인한다. Spring Security의 [CSRF](https://docs.spring.io/spring-security/reference/features/exploits/csrf.html), [Servlet CORS](https://docs.spring.io/spring-security/reference/servlet/integrations/cors.html), [Reactive CORS](https://docs.spring.io/spring-security/reference/reactive/integrations/cors.html) 문서를 참고한다.

요청 형태는 HTTP 경계에서 검증하고 비즈니스 불변식은 도메인이나 유스케이스에서 적용한다. HTTP 응답 매핑은 어댑터에 둔다. `MockMvc` 또는 `WebTestClient`에 보안 테스트 설정을 적용해 실제 보안 체인을 검증한다. 접근 허용과 해당 라우트의 인증 또는 인가 실패를 확인한다. 변경이 CORS나 CSRF 정책에 영향을 주면 관련 사례도 검증한다.
