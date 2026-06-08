import { isAbsolute, normalize } from "node:path";
import { describe, expect, it } from "vitest";
import { feature3SampleScreenshots } from "./feature3Sample";

describe("feature3SampleScreenshots", () => {
  it("uses safe relative screenshot paths for public sample data", () => {
    for (const screenshot of feature3SampleScreenshots) {
      const normalized = normalize(screenshot.filePath);

      expect(isAbsolute(screenshot.filePath), screenshot.filePath).toBe(false);
      expect(normalized.startsWith(".."), screenshot.filePath).toBe(false);
      expect(normalized, screenshot.filePath).toMatch(
        /^\d{4}-\d{2}-\d{2}[\\/]\d{2}-\d{2}\.jpg$/,
      );
    }
  });
});
