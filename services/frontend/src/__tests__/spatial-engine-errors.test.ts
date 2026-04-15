import { beforeEach, describe, expect, it } from "vitest";
import { SpatialEngineAdapter } from "@/lib/wasm/spatial-engine";

/**
 * To test error isolation, we need to call handleMessage directly.
 * handleMessage is private, so we access it via a test subclass.
 */
class TestableAdapter extends SpatialEngineAdapter {
  /** Expose handleMessage for testing. */
  simulateMessage(msg: unknown): void {
    // biome-ignore lint: test-only access to private method
    (this as any).handleMessage(msg);
  }

  /** Expose pending map size for assertions. */
  get pendingCount(): number {
    // biome-ignore lint: test-only access to private field
    return (this as any).pending.size;
  }
}

describe("SpatialEngineAdapter error isolation", () => {
  let adapter: TestableAdapter;

  beforeEach(() => {
    adapter = new TestableAdapter();
    adapter.registerLoadedLayers({ geology: 133, landform: 370 });
  });

  it("query-error rejects ONLY the matching pending request, others survive", async () => {
    const promise1 = new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(1, { kind: "query", resolve, reject });
    });
    // promise2 is intentionally not awaited — its survival is verified via pendingCount
    void new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(2, { kind: "query", resolve, reject });
    });

    expect(adapter.pendingCount).toBe(2);

    // Send query-error for id=1 only
    adapter.simulateMessage({
      type: "query-error",
      id: 1,
      error: "layer not found",
    });

    // id=1 should reject
    await expect(promise1).rejects.toThrow("layer not found");

    // id=2 should still be pending (NOT rejected)
    expect(adapter.pendingCount).toBe(1);
  });

  it("stats-error rejects ONLY the matching pending request", async () => {
    const promise1 = new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(10, { kind: "stats", resolve, reject });
    });
    // promise2 is intentionally not awaited — its survival is verified via pendingCount
    void new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(11, { kind: "query", resolve, reject });
    });

    adapter.simulateMessage({
      type: "stats-error",
      id: 10,
      error: "compute failed",
    });

    await expect(promise1).rejects.toThrow("compute failed");
    expect(adapter.pendingCount).toBe(1); // id=11 survives
  });

  it("catch-all error rejects ALL pending (init-level only)", async () => {
    const promise1 = new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(1, { kind: "query", resolve, reject });
    });
    const promise2 = new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(2, { kind: "query", resolve, reject });
    });

    adapter.simulateMessage({ type: "error", message: "init failed" });

    await expect(promise1).rejects.toThrow("init failed");
    await expect(promise2).rejects.toThrow("init failed");
    expect(adapter.pendingCount).toBe(0);
  });

  it("computeStats returns null in Phase 1 when worker is not ready", async () => {
    const result = await adapter.computeStats({
      south: 35.5,
      west: 139.5,
      north: 35.9,
      east: 140.0,
    });
    expect(result).toBeNull();
  });
});
