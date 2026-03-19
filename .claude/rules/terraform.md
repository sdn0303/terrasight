# Terraform Rules

## Design Principles

| Principle | Description |
| ------------- | ----------- |
| Encapsulation | Group resources that are deployed together into modules |
| Privileges | Separate modules along privilege boundaries (network vs app vs IAM) |
| Volatility | Separate long-lived infrastructure from short-lived resources |
| Least Privilege | Grant minimum required permissions; one service account per service |
| Immutability | Treat infrastructure as code; no manual console changes |

## File Organization

| File | Purpose |
| --------------- | ------- |
| `main.tf` | Primary resources and data sources |
| `variables.tf` | All input variables (alphabetical) |
| `outputs.tf` | All outputs (alphabetical) |
| `terraform.tf` | `required_version` + `required_providers` |
| `providers.tf` | Provider configurations (default first, then aliases) |
| `backend.tf` | State backend configuration |
| `locals.tf` | Local values (or top of file if file-specific) |

- Group resources by purpose: `network.tf`, `iam.tf`, `storage.tf` — not one file per resource
- Define data sources before the resources that reference them
- Use `#` for comments; `//` and `/* */` are not idiomatic

## Naming Conventions

| Target | Convention | Example |
| -------------------- | ---------- | ------- |
| Resources / Variables | snake_case, descriptive noun, no type repeat | `aws_instance.web` |
| Single resource | Use `main` or `this` | `google_storage_bucket.main` |
| Variable units | Include unit suffix | `ram_size_gb`, `disk_size_gib` |
| Booleans | Positive names | `enable_logging` not `disable_logging` |

## Variables and Outputs

- Every variable MUST have `type` and `description`; order: type, description, default, sensitive, validation
- Every output MUST have `description`; reference resource attributes, not input variables
- Provide `default` for environment-independent vars; omit for environment-specific (force explicit)
- Mark sensitive values with `sensitive = true`; use a secret manager for actual secrets
- Use `validation` blocks only when values have uniquely restrictive requirements

## Resources

- Argument order: meta-arguments, then arguments, then nested blocks, then `lifecycle`, then `depends_on`
- `count` for conditionals (`var.enable_x ? 1 : 0`); `for_each` when individual values differ
- `depends_on` as last resort; prefer implicit dependencies via resource references
- `prevent_destroy = true` for stateful resources (databases, storage)
- Run `terraform fmt -recursive` before every commit

## Modules

- One concern per module; if purpose is hard to explain, module is too complex
- Local modules in `./modules/<module_name>/`; pin registry module versions (`~> 1.0`)
- Every module MUST have `README.md`; keep root modules under 100 resources

## State Management

| Rule | Description |
| -------------- | ----------- |
| Remote backend | S3/GCS bucket with versioning and locking; never local state |
| Isolation | One state file per environment; separate directories, not workspaces |
| Lock file | Always commit `.terraform.lock.hcl` |
| Version pinning | Pin Terraform binary, providers, and module versions |

## Security

| Rule | Implementation |
| -------------------- | -------------- |
| No secrets in code | Use secret manager or environment variables; `sensitive = true` |
| IAM least privilege | Minimum permissions per resource; one service account per service |
| State access | Restrict backend bucket to CI/CD; enable audit logging |

## Anti-patterns (Must Fix)

| Anti-pattern | Fix |
| -------------------------- | ------------------------------------------------- |
| Secrets in code / tfvars | Use secret manager; mark variables `sensitive` |
| Local state | Use remote backend with versioning and locking |
| One file per resource | Group by purpose (`network.tf`, `iam.tf`) |
| Hardcoded values | Extract to variables or locals |
| No descriptions | Add `description` to all variables and outputs |
| `depends_on` overuse | Use implicit dependencies via resource references |
| Large root modules | Split into modules (under 100 resources) |
| Manual console changes | Import existing resources or recreate via code |

## Code Review Checklist

- [ ] `terraform fmt` and `terraform validate` pass
- [ ] All variables and outputs have `type` and `description`
- [ ] Sensitive data uses secret manager; `sensitive = true` set
- [ ] State stored remotely with versioning and locking
- [ ] `prevent_destroy` on stateful resources
- [ ] Provider and module versions pinned
- [ ] `.terraform.lock.hcl` committed
- [ ] Modules have README.md
