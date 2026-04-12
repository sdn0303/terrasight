# Validation (Zod)

## Contents

- [Validation (Zod)](#validation-zod)
  - [Contents](#contents)
  - [Core Pattern](#core-pattern)
  - [API Response Validation](#api-response-validation)
  - [React Hook Form Integration](#react-hook-form-integration)
  - [Server Action Validation](#server-action-validation)
  - [Environment Variables](#environment-variables)
  - [Best Practices](#best-practices)

---

## Core Pattern

Define schema once, derive TypeScript type. Never maintain duplicate interfaces.

```typescript
const TransactionSchema = z.object({
  id: z.string().uuid(),
  areaCode: z.string().regex(/^\d{5}$/, 'Area code must be 5 digits'),
  pricePerSqm: z.number().int().positive(),
  transactionDate: z.coerce.date(),
  propertyType: z.enum(['residential', 'commercial', 'industrial']),
  location: z.object({
    lat: z.number().min(-90).max(90),
    lng: z.number().min(-180).max(180),
  }),
});

export type Transaction = z.infer<typeof TransactionSchema>;
```

## API Response Validation

Always validate external data at boundaries with `safeParse`:

```typescript
const GeoJSONResponseSchema = z.object({
  type: z.literal('FeatureCollection'),
  features: z.array(z.object({
    type: z.literal('Feature'),
    geometry: z.object({
      type: z.enum(['Point', 'Polygon', 'MultiPolygon']),
      coordinates: z.unknown(),
    }),
    properties: z.record(z.unknown()),
  })),
});

const result = GeoJSONResponseSchema.safeParse(apiResponse);
if (!result.success) {
  console.error('Invalid GeoJSON:', result.error.flatten());
  return null;
}
```

## React Hook Form Integration

```typescript
const FilterSchema = z.object({
  areaCode: z.string().min(1, 'Area code is required'),
  priceMin: z.coerce.number().min(0).optional(),
  priceMax: z.coerce.number().positive().optional(),
  propertyType: z.enum(['all', 'residential', 'commercial']).default('all'),
});

type FilterValues = z.infer<typeof FilterSchema>;

export function FilterForm() {
  const { register, handleSubmit, formState: { errors } } = useForm<FilterValues>({
    resolver: zodResolver(FilterSchema),
  });
}
```

## Server Action Validation

```typescript
'use server';

export async function createFilter(prevState: unknown, formData: FormData) {
  const raw = Object.fromEntries(formData.entries());
  const result = FilterSchema.safeParse(raw);

  if (!result.success) {
    return { errors: result.error.flatten().fieldErrors };
  }

  return { success: true, data: result.data };
}
```

## Environment Variables

```typescript
const envSchema = z.object({
  REINFOLIB_API_KEY: z.string().min(1),
  DATABASE_URL: z.string().url(),
  NODE_ENV: z.enum(['development', 'test', 'production']).default('development'),
  PORT: z.coerce.number().default(3000),
});

export const env = envSchema.parse(process.env);
```

## Best Practices

- Prefer `safeParse` over `parse` (returns Result, no try/catch needed)
- Use `z.coerce` for FormData and URLSearchParams
- Co-locate schemas with features (`src/features/*/schemas/`)
- Use `.flatten()` or `.format()` for serializable error objects
- `z.record()` rejects `null` — return `{}` not `null` from backend
- Frontend Zod schema is the API contract source of truth
