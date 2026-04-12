---
paths:
  - ".github/**"
---

# GitHub Actions Rules

## Design Principles

- **Reproducibility**: Pin all versions (actions, runners, dependencies) for deterministic builds
- **Least Privilege**: Request minimum `permissions` per workflow; never use default wide-ranging tokens
- **Fast Feedback**: Fail fast with lint/type-check before expensive test/build steps
- **DRY**: Extract shared logic into reusable workflows or composite actions

## Workflow Structure

- File naming: `ci.yml`, `deploy.yml`, `release.yml` -- descriptive, lowercase with hyphens
- Trigger only necessary events: `push` to main, `pull_request` to main, `workflow_dispatch` for manual
- Use `concurrency` to cancel in-progress runs on same branch: `concurrency: { group: ${{ github.workflow }}-${{ github.ref }}, cancel-in-progress: true }`
- Set explicit `timeout-minutes` on every job (30 min recommended; never rely on 6h default)
- Filter paths with `paths` / `paths-ignore` to skip irrelevant workflows

## Job Design

- Use `needs` to define job dependencies: lint -> test -> build -> deploy
- **Matrix strategy**: Test across versions/OS with `strategy.matrix`. Use `fail-fast: false` when collecting results
- Keep jobs focused: one concern per job (lint, test, build, deploy)
- Use `if: always()` or `if: failure()` for cleanup/notification jobs
- Set `runs-on` to specific runner labels (e.g., `ubuntu-24.04`, not `ubuntu-latest`)

## Reusable Workflows

- Define with `on: workflow_call` with typed `inputs` and `secrets`
- Call with `uses: ./.github/workflows/reusable.yml` or `org/repo/.github/workflows/reusable.yml@v1`
- Use `secrets: inherit` within same organization for implicit secret passing
- Prefer reusable workflows for multi-job pipelines; composite actions for single-step reuse

## Caching

- Cache package manager dependencies with `actions/cache` or built-in setup actions
- Key pattern: `${{ runner.os }}-${{ hashFiles('**/lockfile') }}`
- Use `restore-keys` for fallback to partial cache matches
- Cache build artifacts (Go build cache, Next.js `.next/cache`, Docker layers)

## Security

- **Pin actions to SHA**: `uses: actions/checkout@<full-sha>` (never floating tags like `@v4`)
- **Explicit permissions**: Set `permissions: {}` at workflow level, grant per-job
- **OIDC for cloud**: Use `id-token: write` permission with cloud provider OIDC (no long-lived credentials)
- **Secrets**: Never echo secrets. Use `add-mask` for dynamic values. Rotate regularly
- **Supply chain**: Enable Dependabot for action version updates. Use artifact attestations
- **Pull request safety**: Avoid `pull_request_target` with checkout of PR code (code injection risk)
- Restrict workflow modifications with `CODEOWNERS` on `.github/workflows/`

## Artifacts & Outputs

- Use `actions/upload-artifact` / `actions/download-artifact` for cross-job data
- Set `retention-days` explicitly (default 90 days). Minimize artifact size
- Use job `outputs` for passing small values between jobs

## CI Pipeline Pattern

```yaml
# Standard pipeline order:
# 1. lint (biome/ruff/golangci-lint) -- fastest, fail first
# 2. type-check (tsc --noEmit / mypy) -- catch type errors early
# 3. test (vitest/pytest/go test) -- unit + integration
# 4. build (docker build / binary) -- only after tests pass
# 5. deploy (staging -> production) -- only on main branch, with environment protection
```

## Anti-patterns

- **`ubuntu-latest`**: Pin specific versions (`ubuntu-24.04`) for reproducibility
- **Floating action tags (`@v4`)**: Pin to full commit SHA
- **No timeout**: Always set `timeout-minutes` to catch hung jobs
- **Secrets in logs**: Never `echo ${{ secrets.* }}`; use masking
- **Monolithic workflow**: Split into focused jobs with `needs` dependencies
