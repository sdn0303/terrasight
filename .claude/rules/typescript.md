---
paths:
  - "**/*.ts"
  - "**/*.tsx"
  - "services/frontend/**"
---

# TypeScript Rules

## Compiler Settings (tsconfig.json)

**Required strict options** (all `true`):

| Option | Purpose |
|--------|---------|
| `strict` | Enable all strict checks (mandatory) |
| `noUncheckedIndexedAccess` | Add `undefined` to index access |
| `exactOptionalPropertyTypes` | Forbid `undefined` assignment to optional props |
| `verbatimModuleSyntax` | Explicit type imports |
| `isolatedModules` | Single-file transpile compatibility |
| `noUnusedLocals` | Detect unused variables |
| `noUnusedParameters` | Detect unused parameters |
| `noFallthroughCasesInSwitch` | Require break in switch |

**Additional settings**:

- `target`: `esnext`, `module`: `nodenext`, `moduleResolution`: `nodenext`
- `moduleDetection`: `force` (treat all files as modules)
- `noEmit`: `true` (type checking only, bundler handles emit)
- `skipLibCheck`: `true` (faster builds)

## Type Design Principles

| Principle | Description |
|-----------|-------------|
| Derive from values | DRY: use `typeof`, `keyof`, `ReturnType` instead of duplicating |
| Narrow public API | Strict types for exports, flexible for internals |
| `any` is debt | Last resort only, always comment reason |
| Start with `unknown` | External input -> `unknown` -> narrowing |
| Validate at boundary | Use Zod/schema at API/JSON boundaries |

## Type Usage

### `interface` vs `type`

- Object types: `interface` (extendable, clearer errors)
- Union/Intersection/Utility/Mapped types: `type`
- Function types: `type`

### Boundary Types

- `unknown`: External input (API, JSON) -- safe container, narrow before use
- `never`: Exhaustive check, unreachable code
- `any`: Deprecated (legacy migration only, require TODO + issue link)

### Type Assertion Rules

- `as` only at boundaries with validation (Zod)
- Prefer narrowing over assertion
- DOM access exception: `as HTMLElement` acceptable

## Type Patterns

- **Discriminated Union**: Tag property for branching: `{ success: true; data: T } | { success: false; error: E }`
- **Branded Type**: `type UserId = string & { readonly _brand: 'UserId' }`
- **Result Pattern**: `{ ok: true; value: T } | { ok: false; error: E }` with `Ok(value)`, `Err(error)` helpers
- **Immutability**: `readonly` for shallow, `as const` for deep. `Readonly<T>`, `DeepReadonly<T>`
- **Exhaustive Check**: `assertNever(value: never): never` in switch default -- compiler error on new case
- **Widening Prevention**: `as const` for literal preservation, `satisfies` for type check + literal retention
- **`satisfies` Operator**: Validate a value conforms to a type while preserving its narrower literal type: `const cfg = { ... } satisfies Config`
- **`using` Declarations**: Deterministic resource cleanup via `Symbol.dispose`; prefer over manual `try/finally` for DB connections, file handles, timers
- **Runtime Validation (Zod)**: Define schema, derive type with `z.infer<typeof Schema>`. `parse()` throws, `safeParse()` returns Result

## Naming Conventions

| Target | Convention | Example |
|--------|------------|---------|
| Files | kebab-case | `user-service.ts` |
| Types/Interfaces | PascalCase | `UserService`, `ApiResponse` |
| Functions/Variables | camelCase | `getUserById`, `userCount` |
| Constants | UPPER_SNAKE_CASE | `MAX_RETRY_COUNT` |
| Type params | T, TData, TError | Descriptive when needed |

## Anti-patterns

| Anti-pattern | Problem | Fix |
|--------------|---------|-----|
| Implicit `any` | Type safety lost | Enable `noImplicitAny` (included in `strict`) |
| `as` abuse | Hides type errors | Boundary only + Zod validation |
| `!` non-null assertion | Runtime crash risk | Use narrowing / optional chaining |
| `@ts-ignore` | Ignores real errors | Fix the actual issue |
| `@ts-expect-error` | Suppresses errors | Test code only (`.test-d.ts`) |
| `Object`, `{}` | Meaningless types | Use `Record<string, unknown>` |
| `Function` | Disguised any | Use specific function type |

### Exceptions (require comment explaining reason)

- `any`: Legacy migration with TODO + issue link
- `as`: External lib type deficiency (comment lib version)
- `@ts-expect-error`: Intentional test for type errors in `.test-d.ts`
