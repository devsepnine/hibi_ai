---
name: code-hexagonal-kotlin-developer
description: "코틀린 헥사고날 아키텍처, 포트·어댑터, ArchUnit과 Spring Security를 설계하고 검토한다."
---

# Kotlin 헥사고날 아키텍처

## 현재 코드의 경계 파악

1. 빌드 파일, 패키지 구조, 주변 테스트를 읽는다. 실제 호출 하나를 유스케이스부터 저장소나 외부 시스템까지 따라간다.
2. 비즈니스 동작과 그에 필요한 의존성을 찾는다. 프로젝트 관례에 따라 도메인이나 유스케이스에 비즈니스 규칙을 둔다. 외부 시스템과의 경계에 계약이 필요하면 사용하는 코드 가까이에 아웃바운드 포트를 정의한다. 기존 진입점의 계약이 유스케이스에 맞으면 재사용한다.
3. HTTP, 영속화, 외부 연동 세부 사항은 어댑터에 둔다. 전송 데이터와 영속화 모델은 각 어댑터에서 변환한다. 의존성 조립은 프로젝트의 구성 진입점에서 처리하고 도메인과 애플리케이션 코드가 어댑터에 의존하지 않도록 한다.
4. 현재 프로젝트에 맞춰 패키지나 모듈 경계를 정한다. 점진적으로 적용할 때는 변경한 경계부터 검증하고 호출부를 옮기면서 검증 범위를 넓힌다.

Spring HTTP 경계를 다룰 때는 [MVC와 WebFlux의 Spring Security](references/spring-security-ko.md)를 참고한다.

## 의존성 방향 검증

기존 ArchUnit 설정이 있으면 재사용한다. 프로젝트의 패키지 식별자로 규칙을 작성하고 규칙이 적용되는 각 모듈의 클래스 파일을 불러온다. 도메인, 애플리케이션, 어댑터 패키지를 쓰는 프로젝트에서는 아래와 같이 의존성 방향을 검증할 수 있다.

```kotlin
val productionClasses = ClassFileImporter()
    .withImportOption(ImportOption.Predefined.DO_NOT_INCLUDE_TESTS)
    .importPackages("example.app")

noClasses()
    .that().resideInAnyPackage("..domain..", "..application..")
    .should().dependOnClassesThat().resideInAPackage("..adapter..")
    .check(productionClasses)
```

불러온 클래스에 보호 대상과 검증에 필요한 어댑터가 포함되는지 확인한다. 새 규칙을 도입할 때는 테스트 픽스처나 임시 로컬 변경으로 대표적인 금지 의존성이 검증에 실패하는지 확인한다. 관련 유스케이스와 어댑터 테스트는 평소 사용하는 진입점을 통해 동작을 검증한다. 규칙과 클래스 불러오기 옵션은 [ArchUnit 사용자 가이드](https://www.archunit.org/userguide/html/000_Index.html)를 참고한다.
