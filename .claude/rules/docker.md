---
paths:
  - "**/Dockerfile*"
  - "**/docker-compose*"
  - "**/.dockerignore"
---

# Docker Rules

## Design Principles

| Principle | Description |
| --------------- | ----------- |
| Immutable | Config changes require rebuild/redeploy, no runtime modifications |
| Ephemeral | Containers can be stopped, destroyed, and rebuilt with minimal setup |
| Single Concern | One process per container; decouple applications into separate containers |
| Least Privilege | Minimal packages, non-root execution, no unnecessary capabilities |
| Reproducibility | Pin dependency versions and base image tags for deterministic builds |

## Base Images

| Use Case | Image | Notes |
| --------------- | ------------------------------------------------ | -------------------- |
| Static binary | `gcr.io/distroless/static-debian12:nonroot` | Smallest, no shell |
| Binary + glibc | `gcr.io/distroless/base-debian12:nonroot` | Includes glibc |
| Python | `python:3.12-slim-bookworm` | Lightweight Debian |
| Node.js | `node:22-slim` | Lightweight Debian |
| General minimal | `alpine:3.21` | Under 6 MB |

- Use Docker Official, Verified Publisher, or Docker-Sponsored Open Source images
- Prefer distroless for production; slim variants over full images
- Pin specific version tags; consider digest pinning (`FROM image@sha256:...`)
- Rebuild regularly with `docker build --pull` to pick up security patches

## Multi-stage Builds

- Always use multi-stage: full SDK for build, slim/distroless for production
- Name stages with `AS <name>` for clarity and `--target` debugging
- Copy only necessary build artifacts to the final stage
- Create reusable stages for shared components across related images

## Layer Optimization

Order from least to most frequently changing:

1. Base image and system packages
2. Dependency manifests (`package.json`, `requirements.txt`, `go.mod`)
3. Dependency installation
4. Source code copy
5. Build command

- Copy dependency files first, install, then copy source (cache deps across builds)
- Combine `RUN` commands with `&&`; clean caches in the same layer
- Sort multi-line `RUN` arguments alphabetically
- Use `--mount=type=cache` to persist package manager caches across builds
- Use `--mount=type=bind` to temporarily mount files without persisting in layers

## Key Instructions

| Instruction | Rule |
| ----------- | ---- |
| FROM | Specific tags; digest pinning for supply chain security |
| COPY | Prefer over ADD unless extracting tar or fetching remote URL |
| RUN | Combine with `&&`; prepend `set -o pipefail &&` before pipes |
| USER | Switch to non-root before ENTRYPOINT/CMD |
| WORKDIR | Absolute paths only; never `cd` in RUN |
| ENTRYPOINT | Exec form `["executable"]` for the main command |
| CMD | Exec form `["param"]` for default arguments to ENTRYPOINT |
| ENV | PATH updates and version numbers only; never secrets |
| LABEL | Add `version`, `maintainer`, `description`; use EXPOSE for ports |

- Use heredoc (`RUN <<EOF ... EOF`) for multi-command sequences without chaining

## Security

| Rule | Implementation |
| ------------------- | -------------- |
| Non-root user | `USER nonroot:nonroot` or create dedicated user |
| No secrets in image | Never `ENV SECRET=...` or `COPY credentials.json`; use `--mount=type=secret` |
| Minimal packages | `--no-install-recommends`; remove cache in same RUN |
| Read-only rootfs | `--read-only` flag at runtime |
| Version pinning | Specific tags on all base images; never `latest` |
| Image scanning | Trivy / Grype in CI pipeline |

## Anti-patterns (Must Fix)

| Anti-pattern | Fix |
| ----------------------------- | ----------------------------------------------- |
| Root user | Add `USER nonroot` before ENTRYPOINT |
| Secrets in ENV/COPY | Use `--mount=type=secret` or external secret manager |
| `COPY . .` before deps | Copy dependency files first, install, then copy source |
| `apt-get update` alone | Combine with install + cache cleanup in one RUN |
| Full base images | Use slim or distroless variants |
| `latest` tag | Pin specific version tags |
| Missing .dockerignore | Configure to exclude `.git`, `node_modules`, `.env`, docs |
| Pipe without `pipefail` | Prepend `set -o pipefail &&` |

## Code Review Checklist

- [ ] Non-root user execution
- [ ] Multi-stage build with slim/distroless production image
- [ ] No secrets in image (ENV, COPY, or build args)
- [ ] .dockerignore configured
- [ ] Base image and dependency versions pinned
- [ ] Layer order optimized for cache
- [ ] `set -o pipefail` used in piped RUN commands
- [ ] SIGTERM / graceful shutdown handling
