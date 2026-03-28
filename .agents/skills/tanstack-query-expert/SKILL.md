---
name: tanstack-query-expert
description: "Expert in TanStack Query (React Query) v5 — query key factories, mutations, optimistic updates, cache invalidation, and Next.js App Router SSR hydration. Use when implementing data fetching hooks."
metadata:
  version: "1.0.0"
  filePattern:
    - "src/features/*/api/**"
    - "src/hooks/use*Query*"
    - "src/hooks/use*Mutation*"
---

# TanStack Query Expert

Production-grade TanStack Query v5 patterns for this project.

## Project Defaults

```typescript
// Global defaults (set in QueryClientProvider)
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60_000,    // 1 minute
      gcTime: 300_000,      // 5 minutes
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});
```

## Query Key Factory (Required Pattern)

Every feature module MUST define a query key factory:

```typescript
export const transactionKeys = {
  all: ['transactions'] as const,
  lists: () => [...transactionKeys.all, 'list'] as const,
  list: (filters: TransactionFilters) => [...transactionKeys.lists(), filters] as const,
  details: () => [...transactionKeys.all, 'detail'] as const,
  detail: (id: string) => [...transactionKeys.details(), id] as const,
};
```

## Custom Hook Pattern (Required)

Always wrap useQuery in custom hooks:

```typescript
export const useTransactions = (filters: TransactionFilters) => {
  return useQuery({
    queryKey: transactionKeys.list(filters),
    queryFn: () => fetchTransactions(filters),
    staleTime: 1000 * 60 * 5,
    enabled: !!filters.areaCode,
  });
};
```

## Mutation with Cache Invalidation

```typescript
export const useCreateTransaction = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: createTransaction,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: transactionKeys.lists() });
    },
  });
};
```

## Optimistic Updates

```typescript
export const useUpdateTransaction = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: updateTransaction,
    onMutate: async (updated) => {
      await queryClient.cancelQueries({ queryKey: transactionKeys.detail(updated.id) });
      const previous = queryClient.getQueryData(transactionKeys.detail(updated.id));
      queryClient.setQueryData(transactionKeys.detail(updated.id), updated);
      return { previous };
    },
    onError: (_err, _updated, context) => {
      queryClient.setQueryData(transactionKeys.detail(context?.previous?.id), context?.previous);
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: transactionKeys.lists() });
    },
  });
};
```

## Next.js App Router SSR Hydration

```typescript
// Server Component (page.tsx)
export default async function TransactionsPage() {
  const queryClient = new QueryClient();
  await queryClient.prefetchQuery({
    queryKey: transactionKeys.list({ areaCode: '13101' }),
    queryFn: () => fetchTransactionsServer({ areaCode: '13101' }),
  });
  return (
    <HydrationBoundary state={dehydrate(queryClient)}>
      <TransactionsList />
    </HydrationBoundary>
  );
}
```

## Anti-patterns

- Never use `useEffect` to fetch data when TanStack Query is available
- Never sync query data into local state (`useEffect(() => setState(data), [data])`)
- Never use inline `queryFn` without custom hook wrapper
- Never omit `queryKey` factory — use the factory pattern above
