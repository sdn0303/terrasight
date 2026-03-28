---
name: rust-backend-rules
description: "Comprehensive Rust coding guidelines for Axum/Tokio/SQLx backends. 179 rules across ownership, error handling, memory optimization, API design, async patterns, testing, linting, and anti-patterns. Use when writing, reviewing, or refactoring Rust code, Cargo settings, or services/backend code."
metadata:
  version: "1.0.0"
  filePattern:
    - "**/*.rs"
    - "**/Cargo.toml"
    - "**/Cargo.lock"
    - "services/backend/**"
---

# Rust Backend Rules

Production Rust guidance for this repository. The full source rulebook is copied verbatim in [references/rust-rules.md](references/rust-rules.md); keep `SKILL.md` lean and open the reference file only for the categories you need.

## When to Apply

- Writing or refactoring Rust in `services/backend`
- Reviewing Rust diffs for correctness, performance, and maintainability
- Designing error types, async flows, or public APIs
- Changing `Cargo.toml`, profiles, linting, or project structure
- Investigating clone pressure, allocation cost, or borrow checker friction

## Repository Invariants

- No `.unwrap()` in non-test code; prefer `?` or `expect("invariant")`
- Keep `src/domain/` pure with zero external dependencies
- Prefer borrowing over cloning; accept `&str` / `&[T]` over `&String` / `&Vec<T>`
- Use explicit error types in reusable/library layers; allow `anyhow` only at app boundary
- Verify with `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`

## Priority Order

| Priority | Category | Prefix | Notes |
|----------|----------|--------|-------|
| 1 | Ownership & Borrowing | `own-` | Remove unnecessary clones, design for slices/borrows first |
| 2 | Error Handling | `err-` | Propagate context, avoid panics in production |
| 3 | Memory Optimization | `mem-` | Cut allocations only after measuring |
| 4 | API Design | `api-` | Clear types, builders, trait boundaries, DTO shape |
| 5 | Async/Await | `async-` | Avoid holding locks/borrows across `.await` |
| 6 | Compiler Optimization | `opt-` | Profile first; prefer evidence over folklore |
| 7 | Naming Conventions | `name-` | Keep APIs readable and idiomatic |
| 8 | Type Safety | `type-` | Encode invariants in types |
| 9 | Testing | `test-` | Cover success path, failure path, and regressions |
| 10 | Documentation | `doc-` | Document panics, errors, and public contracts |
| 11 | Performance Patterns | `perf-` | Focus on hot paths and measurable impact |
| 12 | Project Structure | `struct-` | Keep module boundaries intentional |
| 13 | Clippy & Linting | `lint-` | Treat warnings as design feedback |
| 14 | Anti-patterns | `anti-` | Use during review to catch regressions fast |

## How to Use the Full Rules

1. Pick the relevant prefix from the table above.
2. Open only the matching sections in `references/rust-rules.md`.
3. Apply the rule together with this repo's local constraints from `AGENTS.md`.

Useful search patterns:

- `^# own-` for ownership and borrowing
- `^# err-` for error handling
- `^# async-` for async/Tokio patterns
- `^# anti-` for code review sweeps
- `^# err-no-unwrap-prod` for the repo's no-unwrap invariant

## Minimal Review Checklist

1. Are ownership and lifetime choices minimizing allocations instead of hiding them with `.clone()`?
2. Are failure modes explicit, contextualized, and test-covered?
3. Does async code avoid lock guards, mutable borrows, or heavy CPU work across `.await`?
4. Are public APIs accepting the most general borrowed types possible?
5. Was any performance-related change justified by measurement?
