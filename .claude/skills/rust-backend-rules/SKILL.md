---
name: rust-backend-rules
description: "Rust coding rules for Axum/Tokio/SQLx backends in services/backend. 179 rules split into 14 category files covering ownership, error handling, async, API design, and more. Use when writing, reviewing, or refactoring Rust code, designing error types, async flows, or public APIs."
metadata:
  version: "2.1.0"
  filePattern:
    - "services/backend/**/*.rs"
    - "services/backend/Cargo.toml"
    - "lib/**/*.rs"
---

# Rust Backend Rules

179 rules across 14 categories for production Rust. Each category is a separate
reference file with a table of contents — open only the ones relevant to your task.

## Core Principles

Derived from Effective Rust, Rust API Guidelines, and The Rust Book:

1. **Work with the compiler, not against it** — borrow checker errors signal
   design problems. Refactor (lifetime extension, lifetime reduction, extract
   method) rather than reaching for `.clone()` or `unsafe`.
2. **Encode invariants in types** — use newtypes, enums, and the typestate
   pattern so invalid states are unrepresentable at compile time.
3. **Errors are values, not surprises** — return `Result`, propagate with `?`,
   add context via `thiserror`. Reserve panics for true programmer bugs.
4. **Profile before optimizing** — measure with `criterion`, `cargo flamegraph`,
   or `EXPLAIN ANALYZE` before changing allocation strategy or adding `#[inline]`.
5. **Leverage the tooling ecosystem** — `cargo fmt`, `cargo clippy`, `cargo test`,
   `cargo doc`, `cargo deny` are your quality gates, not optional extras.

## Quick Reference by Task

- **New function / method**: [01-ownership], [02-error-handling], [07-naming]
- **New struct / public API**: [04-api-design], [08-type-safety], [10-documentation]
- **Async code**: [05-async-await], [01-ownership]
- **Error handling**: [02-error-handling], [04-api-design]
- **Memory / allocation**: [03-memory-optimization], [01-ownership], [11-performance]
- **Performance tuning**: [06-compiler-optimization], [03-memory-optimization]
- **Code review**: [14-anti-patterns], [13-clippy-linting]
- **Testing**: [09-testing], [02-error-handling]
- **Project layout**: [12-project-structure], [13-clippy-linting]

## Category References

Each file has a table of contents at the top. Open only the categories you need:

- **CRITICAL**: [01-ownership], [02-error-handling], [03-memory-optimization]
- **HIGH**: [04-api-design], [05-async-await], [06-compiler-optimization]
- **MEDIUM**: [07-naming], [08-type-safety], [09-testing], [10-documentation], [11-performance]
- **LOW**: [12-project-structure], [13-clippy-linting]
- **REFERENCE**: [14-anti-patterns]

## Borrow Checker Tactics

When the borrow checker rejects code, try these before reaching for `.clone()`:

1. **Lifetime extension** — bind a temporary to a `let` variable to extend its
   lifetime to the enclosing block.
2. **Lifetime reduction** — wrap a borrow in `{ ... }` so it drops before the
   conflicting use.
3. **Extract a method** — move the borrowed data access into a separate function
   whose signature makes the lifetimes explicit.
4. **Use `Cow<'_, T>`** — when a function sometimes needs owned data and
   sometimes borrowed, avoid unconditional cloning.

## Async Safety Checklist

Before any `.await` point, verify:

- No `MutexGuard`, `RwLockWriteGuard`, or `RefMut` held across the `.await`
- No mutable borrows that outlive the suspension point
- CPU-heavy work moved to `tokio::task::spawn_blocking`
- File I/O uses `tokio::fs`, not `std::fs`

## Review Checklist

1. Ownership choices minimize allocations (no gratuitous `.clone()`)?
2. Failure modes explicit, contextualized, and test-covered?
3. Async code avoids lock guards / mutable borrows across `.await`?
4. Public APIs accept the most general borrowed types (`&str`, `&[T]`)?
5. Performance changes justified by measurement?
6. New public items have `///` doc comments with examples?
7. `cargo clippy -- -D warnings` passes without suppression?

## Verification

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

[01-ownership]: references/categories/01-ownership.md
[02-error-handling]: references/categories/02-error-handling.md
[03-memory-optimization]: references/categories/03-memory-optimization.md
[04-api-design]: references/categories/04-api-design.md
[05-async-await]: references/categories/05-async-await.md
[06-compiler-optimization]: references/categories/06-compiler-optimization.md
[07-naming]: references/categories/07-naming-conventions.md
[08-type-safety]: references/categories/08-type-safety.md
[09-testing]: references/categories/09-testing.md
[10-documentation]: references/categories/10-documentation.md
[11-performance]: references/categories/11-performance-patterns.md
[12-project-structure]: references/categories/12-project-structure.md
[13-clippy-linting]: references/categories/13-clippy-linting.md
[14-anti-patterns]: references/categories/14-anti-patterns.md
