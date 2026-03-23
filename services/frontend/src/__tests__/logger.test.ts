import { describe, expect, it } from "vitest";
import { logger } from "@/lib/logger";

describe("logger singleton", () => {
  it("is a pino logger instance", () => {
    // pino loggers expose these standard methods
    expect(typeof logger.info).toBe("function");
    expect(typeof logger.warn).toBe("function");
    expect(typeof logger.error).toBe("function");
    expect(typeof logger.debug).toBe("function");
    expect(typeof logger.child).toBe("function");
  });

  it("has a level set", () => {
    const validLevels = [
      "trace",
      "debug",
      "info",
      "warn",
      "error",
      "fatal",
      "silent",
    ];
    expect(validLevels).toContain(logger.level);
  });

  it("child logger inherits parent level", () => {
    const child = logger.child({ module: "test" });
    expect(child.level).toBe(logger.level);
  });

  it("child logger exposes standard log methods", () => {
    const child = logger.child({ module: "test" });
    expect(typeof child.info).toBe("function");
    expect(typeof child.error).toBe("function");
  });
});
