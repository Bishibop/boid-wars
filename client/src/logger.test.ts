import { describe, it, expect, beforeEach, vi } from "vitest";
import { logger, LogLevel } from "./logger";

describe("Logger", () => {
  beforeEach(() => {
    // Reset console mocks
    vi.clearAllMocks();
  });

  it("should log info messages", () => {
    const consoleSpy = vi.spyOn(console, "log").mockImplementation(() => {});

    logger.setLevel(LogLevel.INFO);
    logger.info("test message", { data: 123 });

    expect(consoleSpy).toHaveBeenCalledWith(
      expect.stringContaining("[INFO]"),
      expect.any(String),
      "test message",
      { data: 123 },
    );
  });

  it("should not log debug messages when level is INFO", () => {
    const consoleSpy = vi.spyOn(console, "log").mockImplementation(() => {});

    logger.setLevel(LogLevel.INFO);
    logger.debug("debug message");

    expect(consoleSpy).not.toHaveBeenCalled();
  });
});
