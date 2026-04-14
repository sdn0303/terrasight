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

> **Use `terrasight-server` for logging, metrics, and HTTP tracing.**

- `tracing::{debug,info,warn,error}!` with structured key-value fields
- Subscriber via `terrasight_server::log::init_global_logger`
- HTTP spans via `terrasight_server::http::tracing::trace_layer()`
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

## Documentation (proj-doc-*)

Sources: [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/documentation.html),
[RFC 1574](https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md),
[RFC 1946](https://github.com/rust-lang/rfcs/blob/master/text/1946-intra-rustdoc-links.md),
[rustdoc book](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html),
[Rust by Example](https://doc.rust-lang.org/rust-by-example/meta/doc.html)

### proj-doc-coverage

> **All public items have `///` doc comments. `//!` at every module/crate top.**

| Level | Syntax | Purpose | Guideline |
|-------|--------|---------|-----------|
| Crate | `//!` in `lib.rs` | Ecosystem positioning, Quick Start, Feature Flags | C-CRATE-DOC |
| Module | `//!` in each `.rs` file | Module overview, type relationships | RFC 1574 |
| Item | `///` on struct/fn/trait/enum | Individual API usage | C-EXAMPLE |

Doc comments are not decorative — they are the primary interface for
subagents and future developers reading `.rs` files directly.

### proj-doc-item-structure

> **Standard structure for item-level doc comments.**

```rust
/// One-line summary (shown in search results and module listings).
///
/// Detailed explanation: motivation, use-case, constraints.
/// Do NOT repeat the type signature — rustdoc renders it automatically.
///
/// # Examples
///
/// ```
/// # use terrasight_domain::scoring::tls::compute_tls;
/// let score = compute_tls(&input)?;
/// assert!(score.total > 0.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns [`DomainError::Validation`] if `input` is empty.
///
/// # Panics
///
/// Panics if `index` is out of bounds.
///
/// # Safety
///
/// (unsafe fn only) Caller must ensure the pointer is valid.
pub fn example(input: &str) -> Result<Foo, Error> { /* ... */ }
```

Section order (RFC 1574): **Examples → Panics → Errors → Safety**.
Use plural headings (`# Examples` not `# Example`).
Verbs in third person present (`Returns` not `Return`).

### proj-doc-crate-level

> **Crate `lib.rs` starts with `//!` block: one-line summary, Quick Start, Feature Flags.**

```rust
//! # terrasight-domain
//!
//! Shared domain types, scoring logic, and constants for the Terrasight platform.
//!
//! ## Quick Start
//!
//! ```rust
//! use terrasight_domain::scoring::tls::compute_tls;
//! // ...
//! ```
//!
//! ## Feature Flags
//!
//! | Flag | Default | Description |
//! |------|---------|-------------|
//! | ... | ... | ... |
```

Opening line: no technical details, let the reader judge relevance instantly.

### proj-doc-examples (C-EXAMPLE, C-QUESTION-MARK)

> **Every public item has at least one `# Examples` with `?` error handling.**

- Use `?` — never `unwrap()` in examples (users copy-paste)
- Hide setup with `# ` prefix lines
- Use `assert!` / `assert_eq!` to make examples double as regression tests
- Show **why** to use the API, not just mechanical invocation

```rust
/// # Examples
///
/// ```
/// # use terrasight_geo::spatial::{LayerKind, compute_feature_limit};
/// let limit = compute_feature_limit(LayerKind::LandPrice, 0.01, 14);
/// assert!(limit >= 100);
/// ```
```

Doc test annotations:
- ```` ```no_run ```` — compiles but doesn't execute (DB/network required)
- ```` ```ignore ```` — skips compilation (pseudocode, external deps)
- ```` ```compile_fail ```` — asserts compilation error (type safety demos)

### proj-doc-intra-links (RFC 1946)

> **Use Rust item paths, not HTML paths, for cross-references.**

```rust
/// Returns [`DomainError::Validation`] if the bbox exceeds
/// [`BBOX_MAX_SIDE_DEG`](crate::constants::BBOX_MAX_SIDE_DEG).
///
/// See [`BBox::new`] for constructor validation.
```

Disambiguators for name collisions: `[struct@Foo]`, `[fn@bar]`, `[macro@baz!]`.

### proj-doc-module-level

> **Every `.rs` file begins with `//!` explaining what this module does and why.**

```rust
//! PostgreSQL repository for land price spatial queries.
//!
//! Implements [`LandPriceRepository`](crate::domain::repository::LandPriceRepository)
//! using PostGIS `ST_MakeEnvelope` for bounding-box queries with N+1 truncation.
```

For module index files (e.g., `infra.rs`, `handler.rs`), document the
architectural role and list key submodules.

### proj-doc-visibility

> **Use `pub(crate)` to control API surface — not `#[doc(hidden)]`.**

`pub(crate)` items are excluded from `rustdoc` automatically.
Reserve `#[doc(hidden)]` for re-export control only.

### proj-doc-verification

> **CI must enforce documentation quality.**

```bash
# Build docs with warnings-as-errors (catches broken intra-doc links)
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace

# Run doc-tests
cargo test --doc

# Optional: warn on missing docs
# Add to lib.rs: #![warn(missing_docs)]
```

### proj-doc-antipatterns

> **Avoid these documentation mistakes.**

| Anti-pattern | Fix |
|-------------|-----|
| Repeating the type signature in prose | rustdoc renders it — describe intent |
| `unwrap()` in examples | Use `?` + hidden `fn main()` |
| HTML path links (`struct.Foo.html`) | Intra-doc links (`[`Foo`]`) |
| No `# Examples` on public items | Every public item needs at least one |
| Commenting "what" only (`// increment i`) | Write "why" and "when" |
| `#[doc(hidden)]` on internal items | Use `pub(crate)` visibility |
| README and lib.rs docs diverge | Use `#[doc = include_str!("../README.md")]` |

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
