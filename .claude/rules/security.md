# Security Rules

## Design Principles

- **Defense in Depth**: Multiple layers of security; no single point of failure
- **Least Privilege**: Grant minimum required access to users, services, and processes
- **Zero Trust**: Verify every request. Never trust network location alone
- **Shift Left**: Integrate security checks into development and CI, not just production

## Secrets Management

- **Never commit secrets** to version control. No API keys, passwords, or tokens in code or config files
- Store secrets in a vault (HashiCorp Vault, AWS Secrets Manager, GCP Secret Manager)
- Application config: Use environment variables loaded from vault at runtime, not `.env` files in production
- `.env` files for local development only. Always listed in `.gitignore`
- **Rotation**: Automate secret rotation. Define maximum lifetime for all credentials
- **Detection**: Use secret scanning tools in CI (gitleaks, truffleHog, GitHub secret scanning)
- Language-specific: Go (`os.Getenv`), Python (`pydantic.SecretStr`), Node.js (Zod-validated `process.env`)
- Docker: Use `--mount=type=secret` for build-time secrets. Never `ENV` or `COPY` credentials

## Dependency Management

- **Lock files**: Always commit lockfiles (`pnpm-lock.yaml`, `go.sum`, `uv.lock`)
- **Audit regularly**: `pnpm audit`, `go vuln check`, `pip audit`, `npm audit`
- **Automated updates**: Enable Dependabot or Renovate for dependency PRs
- **Pin versions**: Use exact versions in lockfiles. Review changelogs before updating
- **Minimize dependencies**: Every dependency is an attack surface. Evaluate necessity before adding
- **SBOM**: Generate Software Bill of Materials (CycloneDX/SPDX) in CI pipeline

## Supply Chain Security

- Pin GitHub Actions to full SHA (not floating tags)
- Pin Docker base images to digest (`FROM image@sha256:...`)
- Verify package integrity (checksums, signatures) for critical dependencies
- Use artifact attestations for build provenance (SLSA, Sigstore)

## Input Validation

- Validate all external input at system boundaries (API, CLI, file upload, environment)
- Use schema validation (Zod, Pydantic, Go struct tags) -- never trust client data
- Sanitize for context: HTML escape, SQL parameterization, shell argument quoting
- Reject unexpected input types, sizes, and formats. Fail closed

## Authentication & Authorization

- Use established libraries/services (never roll custom auth)
- Tokens: Short-lived access tokens + refresh tokens. Store securely (httpOnly cookies, not localStorage)
- Passwords: Bcrypt/Argon2 with salt. Never SHA/MD5. Never store plaintext
- RBAC: Define roles with minimum required permissions. Enforce at API layer
- MFA: Require for administrative and privileged operations

## Data Protection

- **In transit**: TLS 1.2+ mandatory for all connections. No plaintext HTTP in production
- **At rest**: Encrypt databases, backups, and file storage. Use managed encryption keys (KMS)
- **PII**: Classify and document personal data. Minimize collection. Apply retention policies
- **Masking**: Mask sensitive data in logs, error messages, and non-production environments

## Logging & Monitoring

- **No PII in logs**: Never log passwords, tokens, PII, or full request bodies with sensitive data
- Structured logging with request ID, user ID (anonymized), operation, and outcome
- Alert on: authentication failures, privilege escalation, rate limit breaches, error spikes
- Retain audit logs for compliance period. Protect logs from tampering

## Incident Response

- Maintain a security incident runbook with escalation contacts
- Post-incident: Write blameless post-mortem. Track remediation items
- Report vulnerabilities via `SECURITY.md` in repository root (coordinated disclosure)

## OWASP Top 10 (2025)

- Review against OWASP Top 10:2025 during design and code review
- Top risks: Broken Access Control, Security Misconfiguration, Supply Chain Failures, Injection
- Conduct periodic threat modeling for critical services

## Anti-patterns

- **Secrets in source code or CI logs**: Use vault + secret scanning
- **Trusting client-side validation**: Always validate server-side
- **Overly permissive CORS** (`*`): Whitelist specific origins
- **Logging sensitive data**: Mask PII, tokens, and credentials
- **Ignoring dependency vulnerabilities**: Audit and update regularly
