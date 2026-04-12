---
paths:
  - "services/backend/**/*.rs"
  - "services/backend/Cargo.toml"
  - "services/wasm/**/*.rs"
  - "lib/**/*.rs"
---

# Rust Rules

Project conventions for Rust code. These override generic Rust guidance.
For the full 179-rule reference, use the `rust-backend-rules` skill.

## Guiding Principles

1. **Work with the compiler** — borrow checker errors are design feedback.
   Refactor lifetimes (extend, reduce, extract method) before adding `.clone()`.
2. **Encode invariants in types** — newtypes, enums, typestate. Invalid states
   should be unrepresentable at compile time.
3. **Errors are values** — `Result` + `?` + `thiserror`. Panics only for
   programmer bugs behind `#[cfg(test)]`.
4. **Profile before optimizing** — `criterion`, `cargo flamegraph`, or
   `EXPLAIN ANALYZE`. No folklore-driven optimization.
5. **Leverage the tooling ecosystem** — `cargo fmt`, `clippy`, `test`, `doc`,
   `deny` are mandatory quality gates.

## Verification

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Project Conventions

### proj-no-mod-rs

> **Eliminate `mod.rs`. Use Rust 2018 file-as-module format.**

```text
# Bad                          # Good
src/domain/mod.rs              src/domain.rs  (pub mod declarations)
src/domain/value_object.rs     src/domain/value_object.rs
```

### proj-telemetry-conventions

> **Use `realestate-telemetry` for logging, metrics, and HTTP tracing.**

- `tracing::{debug,info,warn,error}!` with structured key-value fields
- Subscriber via `realestate_telemetry::log::init_global_logger`
- HTTP spans via `realestate_telemetry::http::trace_layer()`
- No `println!`, `eprintln!`, or direct `env_logger`

### proj-trace-with-inspect

> **Use `.inspect()` / `.inspect_err()` for tracing inside Result chains.**

```rust
let rows = repo
    .find_all(&bbox)
    .await
    .inspect(|r| tracing::debug!(count = r.len(), "rows fetched"))
    .inspect_err(|e| tracing::warn!(error = %e, "fetch failed"))?;
```

### proj-propagate-errors

> **Propagate with `?` to the outermost caller. Only handlers convert errors.**

- Domain/usecase: `Result<T, DomainError>`
- Handler: `DomainError → AppError` via `ErrorMapping`
- No `.unwrap()` outside `#[cfg(test)]` and documented `INVARIANT:` comments
- Use `.inspect_err` for observation, not `match` to rewrap

### proj-types-over-literals

> **Reject raw `String`/`i64`/`f64` at module boundaries.**

- Newtypes or enums with validated constructors
- Serde DTOs convert via `into_domain` / `into_filters`
- Classification values use enums with `FromStr` / `Display`

### proj-method-chains-over-nesting

> **Prefer iterator chains and early returns. Max 4 indentation levels.**

### proj-match-for-complex-conditions

> **Use `match` with tuple patterns for multi-predicate conditions.**

```rust
match (cache.as_ref(), ttl > 0, stale, retries) {
    (Some(value), true, false, _) => return Ok(value.clone()),
    (None, _, _, r) if r < 3 => refresh(retries + 1).await?,
    _ => fallback(),
}
```

### proj-tests-colocated

> **Unit tests in `#[cfg(test)] mod tests` in the same file.**

- Integration tests in `tests/`, no shared `tests/common/`

### proj-clean-architecture

> **Four-layer Clean Architecture. Dependencies point inward only.**

```text
handler → usecase → domain ← infra
```

- `domain`: only `std`, `serde`, `thiserror`, `chrono`, `async-trait`
- `infra`: implements `domain::repository` traits, no leaked framework errors
- `usecase`: orchestration via repository traits
- `handler`: only layer with `axum`/`http`; adapts `DomainError → AppError`

### proj-shared-code-granularity

> **Extract shared code only at 3+ call sites or for layering needs.**

### proj-lib-crate-encapsulation

> **Re-export public symbols from `lib.rs`. `pub(crate)` internals.**

### proj-rust-idiom-naming

> **Idiomatic naming: `GetXUsecase`, `XRepository`, `LandPrice`.**

- Usecases: `Get`/`Compute`/`Create`/`Update`/`Delete` + `Usecase`
- Repos: `find_…`, `get_…`, `calc_…`, `save_…`, `delete_…`
- Domain types: plain nouns. DTOs carry `Dto` suffix at handler boundary
- `HttpClient` not `HTTPClient`

### proj-no-over-abstraction

> **Ship concrete first. Traits only for DI or layer seams.**

- `Arc<dyn Trait>` only at layer boundaries
- Generic type params need 2+ call sites
- Macros are a last resort

## Cargo.toml Profiles

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.dev.package."*"]
opt-level = 3
```
