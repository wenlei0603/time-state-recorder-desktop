import { describe, expect, it, vi } from "vitest";
import { createCollectorFetcher, resolveCollectorUrl } from "./collectorFetch";

describe("collector fetch URL routing", () => {
  it("routes collector API and screenshot paths through the configured API URL", async () => {
    const fetcher = vi.fn(async () => ({
      ok: true,
      status: 200,
      statusText: "OK",
      json: async () => ({})
    }));
    const collectorFetch = createCollectorFetcher("http://127.0.0.1:5317", fetcher);

    await collectorFetch("/api/health");
    await collectorFetch("/screenshots/2026-06-09/10-00.jpg");

    expect(fetcher).toHaveBeenNthCalledWith(1, "http://127.0.0.1:5317/api/health", undefined);
    expect(fetcher).toHaveBeenNthCalledWith(
      2,
      "http://127.0.0.1:5317/screenshots/2026-06-09/10-00.jpg",
      undefined
    );
  });

  it("leaves browser-preview relative paths unchanged when no API URL is known", () => {
    expect(resolveCollectorUrl("/api/health")).toBe("/api/health");
    expect(resolveCollectorUrl("/screenshots/a.jpg")).toBe("/screenshots/a.jpg");
  });

  it("does not rewrite already absolute URLs", () => {
    expect(
      resolveCollectorUrl("https://example.test/api/health", "http://127.0.0.1:5317")
    ).toBe("https://example.test/api/health");
  });
});
