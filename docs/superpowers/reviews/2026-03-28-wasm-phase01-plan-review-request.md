# Codex Review Request: WASM Phase 0+1 Plan (v3)

## CRITICAL: Read the plan file fresh

The plan has been updated to v3. Previous reviews were against v2 and found issues that have been fixed. **You MUST re-read the plan file from disk, not from cache.**

Plan file: `docs/superpowers/plans/2026-03-28-wasm-phase0-phase1.md`

## v3 Changes (verify these exist in the file)

### Fix 1: computeStats unconditional throw
- **Line ~294**: Must contain `throw new Error("WASM stats disabled in Phase 1")`
- **NOT** `_loadedLayers.size === 0` guard
- **Line ~13**: Non-Goals states "computeStats() remains unconditionally throwing"
- Verify: `grep -n "WASM stats disabled" docs/superpowers/plans/2026-03-28-wasm-phase0-phase1.md`

### Fix 2: Behavior test for error isolation (NOT shape test)
- **Line ~399-442**: Must contain `TestableAdapter` class with `simulateMessage()` and `pendingCount`
- **Line ~426-442**: Must show pending 2 requests, `query-error(id=1)`, assert id=1 rejected and id=2 survives
- **NOT** just `const queryError = { type: "query-error" ...}` shape assertions
- Verify: `grep -n "TestableAdapter\|pendingCount\|simulateMessage" docs/superpowers/plans/2026-03-28-wasm-phase0-phase1.md`

### Fix 3: failed_layers in pino log
- **Line ~709**: Must contain `failed_layers: failedLayers`
- Verify: `grep -n "failed_layers" docs/superpowers/plans/2026-03-28-wasm-phase0-phase1.md`

## Review Focus

1. Run the grep commands above to verify v3 content is present
2. Are the 5 prior issues now ALL resolved?
3. Any NEW issues?
4. Are tests sufficient?
5. Is implementation order correct?

## Files to Read

- `docs/superpowers/plans/2026-03-28-wasm-phase0-phase1.md` — THE PLAN (v3, read fresh)
- `docs/superpowers/specs/2026-03-28-wasm-optimization-design.md` — Design spec
- `services/frontend/src/lib/wasm/spatial-engine.ts` — Current adapter
- `services/frontend/src/lib/wasm/worker.ts` — Current worker
- `services/frontend/src/hooks/use-spatial-engine.ts` — Current hook
