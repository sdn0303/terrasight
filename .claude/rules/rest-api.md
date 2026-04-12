---
paths:
  - "services/backend/src/handler/**"
  - "services/frontend/src/app/api/**"
  - "services/frontend/src/features/*/api/**"
---

# REST API Design Rules

## Design Principles

- **Resource-oriented**: URLs represent resources (nouns), not actions (verbs)
- **Stateless**: Each request contains all information needed; no server-side session
- **Consistent**: Uniform naming, error format, and pagination across all endpoints
- **Evolvable**: Design for backward compatibility; use versioning for breaking changes

## URL Design

- Use plural nouns for collections: `/users`, `/orders`, `/order-items`
- Use kebab-case for multi-word paths: `/order-items` (not camelCase or snake_case)
- Nest resources max 2 levels: `/users/{userId}/orders` (not deeper)
- Use path parameters for identity: `/users/{userId}`, query parameters for filtering: `/users?status=active`
- **Versioning**: URI prefix `v1`: `/api/v1/users`. Increment only on breaking changes
- No trailing slashes. No file extensions (`.json`)

## HTTP Methods

| Method | Semantics | Idempotent | Response |
|--------|-----------|------------|----------|
| GET | Retrieve resource(s) | Yes | 200 with body |
| POST | Create new resource | No | 201 with `Location` header |
| PUT | Replace entire resource | Yes | 200 or 204 |
| PATCH | Partial update | No | 200 with updated resource |
| DELETE | Remove resource | Yes | 204 No Content |

## Status Codes

**Success (2xx)**:

- `200 OK`: General success with body. `201 Created`: Resource created. `204 No Content`: Success without body

**Client Error (4xx)**:

- `400 Bad Request`: Malformed syntax. `401 Unauthorized`: Missing/invalid credentials
- `403 Forbidden`: Authenticated but not authorized. `404 Not Found`: Resource does not exist
- `409 Conflict`: State conflict (duplicate). `422 Unprocessable Entity`: Validation failure
- `429 Too Many Requests`: Rate limit exceeded (include `Retry-After` header)

**Server Error (5xx)**:

- `500 Internal Server Error`: Unexpected failure. `503 Service Unavailable`: Temporary overload

## Request & Response

- Use `camelCase` for JSON property names
- Dates in ISO 8601 format: `2025-01-15T09:30:00Z` (always UTC with `Z`)
- No envelope wrapping: return resource directly, not `{ "data": { ... } }`
- Collection responses: return array directly or `{ "items": [...], "nextCursor": "..." }`
- Include `Content-Type: application/json` in all responses
- Use `Accept-Language` header for localization, not URL parameters

## Error Format

Consistent error response across all endpoints (nested envelope):

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "The 'email' field is required.",
    "details": { "email": "field required" }
  }
}
```

- `error.code`: Machine-readable error code (UPPER_SNAKE_CASE)
- `error.message`: Human-readable description
- `error.details`: Optional object with field-level errors or extra context

## Pagination

- Use cursor-based pagination: `?cursor=<opaque>&limit=20`
- Return `nextCursor` in response body (or `null` when no more pages)
- OFFSET-based pagination prohibited for large datasets (degrades with scale)
- Include `Link` header with `rel="next"` for discoverability

## Filtering & Sorting

- Filter via query params: `?status=active&role=admin`
- Sort via `sort` param: `?sort=createdAt` (prefix `-` for descending: `?sort=-createdAt`)
- Search via `q` param: `?q=keyword`

## Security

- Require authentication on all non-public endpoints (Bearer token / API key)
- Apply rate limiting with `429` response and `Retry-After` header
- Set CORS headers explicitly. Never use `Access-Control-Allow-Origin: *` in production
- Validate and sanitize all input. Never trust client data

## Documentation

- Maintain OpenAPI 3.x specification as source of truth
- Generate documentation from spec. Keep spec in version control alongside code

## Anti-patterns

- **Verbs in URLs** (`/getUsers`): Use nouns with HTTP methods
- **Inconsistent naming**: Mix of camelCase/snake_case in URLs or JSON
- **Generic status codes**: Always return specific 4xx/5xx, not just 200 with error body
- **Deeply nested URLs** (`/a/{id}/b/{id}/c/{id}/d`): Flatten or use query params
- **OFFSET pagination on large datasets**: Use cursor-based
