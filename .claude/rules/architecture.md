# Architecture & Design Pattern Rules

## Core Principles

- **Dependency Inversion**: High-level modules must not depend on low-level modules. Both depend on abstractions (interfaces)
- **Separation of Concerns**: Each module/layer has a single, well-defined responsibility
- **Validate at Boundaries**: All external input is validated at system entry points (API, CLI, message consumer)
- **Framework Independence**: Business logic must not depend on frameworks, databases, or delivery mechanisms
- **Explicit over Implicit**: Dependencies are injected, not imported globally or resolved magically

## Architecture Selection Guide

Choose architecture based on **domain complexity** and **team size**, not anticipated scale:

| Complexity | Team | Architecture | Structure |
|------------|------|--------------|-----------|
| Simple CRUD / MVP | 1-3 | 2-layer or MVC | Handler + Repository (or Controller + Model) |
| Standard application | 3-10 | Clean Architecture 4-layer | Handler / Usecase / Domain / Infra (aiden scaffold default) |
| Complex domain | 5-20 | Modular Monolith + DDD | Feature modules with Bounded Contexts |
| Multiple teams / isolation needs | 20+ | Microservices | Independent services + API contracts |

**Decision criteria**:
- Start with the **simplest** architecture that fits. Evolve when pain justifies it
- Monolith-first: Extract services only when organizational or scaling boundaries demand it
- A modular monolith with clear boundaries is preferable to premature microservices
- AI-assisted development favors simpler architectures -- complexity costs more than it saves

## Existing Project Adaptation

**Framework conventions take priority**. Do not force-fit Clean Architecture onto existing projects:

| Framework Pattern | Mapping to Clean Principles | Adaptation |
|-------------------|----------------------------|------------|
| MVC (Rails, Spring) | Controller=Handler, Model=Domain+Infra, View=Presentation | Extract Service layer for complex logic |
| MVT (Django) | View=Handler, Model=Domain+Infra, Template=Presentation | Use Service/Usecase for business logic beyond Model |
| Module (NestJS, Angular) | Module=Bounded Context, Controller=Handler, Service=Usecase | Add Repository interface for data access abstraction |
| Serverless (Lambda) | Handler=Entry point, shared lib=Domain | Keep functions thin; extract shared domain logic |

- **New features** in existing projects: Apply clean principles within the feature boundary
- **Existing code**: Refactor incrementally only when modifying. Never rewrite for architecture alone
- **Respect the codebase**: Follow the project's established naming, directory structure, and patterns

## Layer Architecture (New Project Default)

Default for new projects scaffolded by aiden. See language-specific rules for implementation details.

| Layer | Responsibility | Depends On |
|-------|----------------|------------|
| Handler | I/O (HTTP, gRPC, CLI), request validation, response formatting | Usecase |
| Usecase | Business logic orchestration, transaction management, logging | Domain |
| Domain | Entities, value objects, repository interfaces, business rules | Nothing (pure) |
| Infra | Database, external APIs, file system, message queues | Domain (implements interfaces) |

**Dependency direction**: Handler -> Usecase -> Domain <- Infra

- Domain layer has **zero external dependencies** -- no frameworks, no I/O
- Infra implements Domain interfaces (Repository, Gateway) via dependency injection
- Logging occurs **once** at the Usecase boundary. Lower layers propagate errors only

## Domain-Driven Design (DDD)

Apply DDD tactical patterns when domain complexity is **high** (multiple business rules, complex state transitions). Skip for simple CRUD.

**Strategic Patterns**:
- **Bounded Context**: Define clear boundaries around domain models. Each context has its own ubiquitous language
- **Context Mapping**: Define relationships between contexts (Shared Kernel, Anti-Corruption Layer, Customer-Supplier)

**Tactical Patterns**:
- **Entity**: Has identity that persists across state changes. Equality by ID
- **Value Object**: Immutable, no identity. Equality by attribute values. Prefer over Entity when possible
- **Aggregate**: Cluster of entities/value objects with a root. All changes go through the Aggregate Root
- **Repository**: Persistence abstraction for Aggregate Roots only. One repository per aggregate
- **Domain Event**: Record significant state changes. Use for decoupling between aggregates/contexts
- **Domain Service**: Business logic that does not belong to a single entity (e.g., transfer between accounts)

## Design Patterns

- **Dependency Injection**: Always. Constructor injection preferred. Avoid service locator
- **Repository**: Data access abstraction. One per aggregate root
- **Strategy / Factory**: Strategy for runtime behavior selection; Factory for complex object creation
- **Result / Either**: Error handling without exceptions. Return `{ok, value}` or `{error, reason}`
- **Observer / Event**: Decouple side effects from core logic (notifications, audit, cache invalidation)

## CQRS & Event Sourcing

- **CQRS**: Separate read/write models when patterns diverge significantly. Not for standard CRUD
- **Event Sourcing**: Immutable event log as source of truth. Apply for audit/temporal needs (finance, compliance)
- Events in past tense (`OrderPlaced`, not `PlaceOrder`). Both add significant complexity -- justify before adopting

## Module / Package Design

- **High cohesion, low coupling**: One reason to change per module. Communicate via interfaces or events
- **No circular dependencies**: Extract shared logic into a third module to break cycles
- **Minimal public API**: Export only what consumers need. Use barrel files or package visibility

## Testing Strategy

- **Simple CRUD / MVC**: Favor integration tests (50%) over unit tests (20%). E2E for critical paths (30%)
- **Clean Architecture**: Domain/Usecase unit tests (50%) with mocked repositories. Integration (30%). E2E (20%)
- **Microservices**: Contract tests between services. Unit (40%), Integration (40%), E2E (20%)
- Domain layer: No I/O in tests. Infra: testcontainers for real DB. Handler: HTTP client E2E

## Anti-patterns

- **Premature microservices**: Start monolith-first. Extract when organizational boundaries demand it
- **Anemic domain model**: Entities with only getters/setters and no business logic. Put behavior in the domain
- **God class / God module**: Split into focused, single-responsibility components
- **Circular dependencies**: Extract shared concerns into a separate module
- **Architecture astronautics**: Over-engineering for hypothetical future requirements. Build for today's needs
