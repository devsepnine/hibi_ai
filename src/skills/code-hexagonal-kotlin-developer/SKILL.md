---
name: code-hexagonal-kotlin-developer
description: "Design Kotlin ports and adapters, ArchUnit checks, and Spring security. 코틀린 헥사고날."
---

# Kotlin hexagonal architecture

## Map the current boundaries

1. Read the build files, package layout, and nearby tests. Trace one real caller through the use case to storage or another external system.
2. Identify the business behavior and the dependencies it needs. Put business rules in the domain or use case according to the project's conventions. Define outbound ports near their consumers where a boundary needs a contract. Reuse the existing entry point when it already exposes the right use case contract.
3. Keep HTTP, persistence, and integration details in adapters. Translate transport or persistence representations at those boundaries. Place dependency wiring in the project's composition root and keep domain and application code independent of adapters.
4. Choose package or module boundaries that fit the current project. For gradual adoption, add a check around the changed boundary and grow the protected scope as callers move.

For a Spring HTTP boundary, consult [Spring Security for MVC and WebFlux](references/spring-security.md).

## Verify the dependency direction

Reuse an existing ArchUnit setup where possible. Write rules against package identifiers from the project and import class files from each module the rule governs. For a project that uses domain, application, and adapter packages, an inward dependency rule can look like this.

```kotlin
val productionClasses = ClassFileImporter()
    .withImportOption(ImportOption.Predefined.DO_NOT_INCLUDE_TESTS)
    .importPackages("example.app")

noClasses()
    .that().resideInAnyPackage("..domain..", "..application..")
    .should().dependOnClassesThat().resideInAPackage("..adapter..")
    .check(productionClasses)
```

Check that the import includes classes the rule should protect and the adapters it should recognize. When introducing the rule, verify that a representative forbidden dependency fails the check using a test fixture or a temporary local change. Run the relevant use case and adapter tests to verify behavior through their normal entry points. See the [ArchUnit user guide](https://www.archunit.org/userguide/html/000_Index.html) for rule and import options.
