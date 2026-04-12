# Data Fetching

## Contents

- [Project Defaults](#project-defaults)
- [Query Key Factory](#query-key-factory)
- [Custom Hook Pattern](#custom-hook-pattern)
- [Mutation with Cache Invalidation](#mutation-with-cache-invalidation)
- [Optimistic Updates](#optimistic-updates)
- [SSR Hydration](#ssr-hydration)
- [Cache Policy](#cache-policy)
- [Anti-patterns](#anti-patterns)

---

## Project Defaults

```typescript
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60_000,
      gcTime: 300_000,
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});
```

## Query Key Factory

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

## Custom Hook Pattern

Always wrap `useQuery` in custom hooks:

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
      if (context?.previous) {
        queryClient.setQueryData(
          transactionKeys.detail(context.previous.id),
          context.previous,
        );
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: transactionKeys.lists() });
    },
  });
};
```

## SSR Hydration

```typescript
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

## Cache Policy

- `cacheLife('days')` for master data (area codes, categories)
- `cacheLife('minutes')` for list queries
- `tags` for detail views (invalidate on mutation)
- `revalidate: 0` for real-time data

## Anti-patterns

- `useEffect` to fetch data when TanStack Query is available
- Syncing query data into local state: `useEffect(() => setState(data), [data])`
- Inline `queryFn` without custom hook wrapper
- Missing query key factory — always use the factory pattern
- Debounce-less Zustand `viewState` -> query key (request flood)
