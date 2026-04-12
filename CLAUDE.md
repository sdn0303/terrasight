# Claude Code Configuration

@AGENTS.md

## Rules

Rules in `.claude/rules/` are scoped by `paths:` frontmatter — only loaded
when editing matching files. Three rules are always loaded: `architecture.md`,
`security.md`, `workflow.md`.

## Skills (progressive disclosure)

Each skill has a thin SKILL.md and reference files loaded on demand:

- `rust-backend-rules` — 14 reference files (ownership, errors, async, API design, etc.)
- `frontend-nextjs-rules` — 5 reference files (React patterns, data fetching, state, validation, UI)
- `postgresql-patterns` — 3 reference files (schema design, query optimization, migrations)
- `geospatial-dev` — MapLibre GL + PostGIS integration patterns

## Agents

| Agent | Model | Purpose |
| ----- | ----- | ------- |
| `rust-engineer` | sonnet | Axum/Tokio/SQLx backend implementation |
| `frontend-developer` | sonnet | Next.js 16 / React 19 / MapLibre frontend |
| `database-admin` | sonnet | PostgreSQL/PostGIS schema, queries, migrations |
| `code-reviewer` | opus | Cross-stack code review (Rust + TypeScript) |
| `security-auditor` | opus | Security audit (read-only) |
| `test-automator` | sonnet | Unit, integration, and E2E tests |

## Workflow

See `.claude/rules/workflow.md` for the full development flow including:

- Rust design doc requirement before implementation
- Agent delegation rules with skill/rule references
- Subagent anti-patterns and model selection
