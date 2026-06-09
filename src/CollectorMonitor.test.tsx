import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { CollectorMonitor } from "./CollectorMonitor";
import type { DesktopRuntimeClient } from "./lib/desktopRuntime";

function healthResponse() {
  return {
    ok: true,
    status: 200,
    statusText: "OK",
    json: async () => ({
      status: "ok",
      startedAt: "2026-06-04T00:00:00Z",
      uptimeSeconds: 120,
      version: "1.2.0",
      windowCollector: { status: "running", errorCount: 0 },
      inputCollector: { status: "running", errorCount: 0 },
      screenshotCollector: { status: "running", errorCount: 0 },
      dbStats: {
        windowEvents: 10,
        lifecycleEvents: 0,
        inputEvents: 0,
        textSegments: 0,
        screenshots: 5,
        highResScreenshots: 2,
        blockerHits: 0,
        imageRetention: {
          retentionDays: 30,
          activeFiles: 7,
          expiredFiles: 3,
          activeBytes: 123456,
          expiredBytes: 45678,
          pendingGoogleDriveUpload: true,
          googleDriveMessage:
            "Local screenshots are temporary for 30 days. Upload older evidence to Google Drive before cleanup.",
        },
      },
    }),
  };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("CollectorMonitor image retention", () => {
  it("shows the 30 day local retention and Google Drive upload reminder", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(healthResponse() as Response);

    render(<CollectorMonitor />);

    expect(await screen.findByText("Image Retention")).toBeInTheDocument();
    expect(screen.getByText("30 days local")).toBeInTheDocument();
    expect(screen.getByText(/Upload older evidence to Google Drive/i)).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
  });
});

function createDesktopClient(status = "running", managed = true): DesktopRuntimeClient {
  return {
    getCollectorStatus: vi.fn(async () => ({
      status,
      managed,
      pid: managed ? 1234 : null,
      apiUrl: "http://127.0.0.1:4317",
      dataDir: "D:/TSR/data",
      lastError: null
    })),
    startCollector: vi.fn(async () => ({
      status: "running",
      managed: true,
      pid: 1235,
      apiUrl: "http://127.0.0.1:4317",
      dataDir: "D:/TSR/data",
      lastError: null
    })),
    stopCollector: vi.fn(async () => ({
      status: "stopped",
      managed: false,
      pid: null,
      apiUrl: "http://127.0.0.1:4317",
      dataDir: "D:/TSR/data",
      lastError: null
    })),
    listenRuntimeEvent: vi.fn(async () => () => undefined)
  };
}

describe("CollectorMonitor desktop controls", () => {
  it("pauses and resumes a desktop-managed collector", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(healthResponse() as Response);
    const client = createDesktopClient("running", true);

    render(<CollectorMonitor desktopClient={client} />);

    fireEvent.click(await screen.findByRole("button", { name: /pause capture/i }));
    await waitFor(() => expect(client.stopCollector).toHaveBeenCalledTimes(1));
    expect(await screen.findByRole("button", { name: /resume capture/i })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /resume capture/i }));
    await waitFor(() => expect(client.startCollector).toHaveBeenCalledTimes(1));
    expect(await screen.findByRole("button", { name: /pause capture/i })).toBeInTheDocument();
  });

  it("does not offer pause for an external collector owned outside the desktop app", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(healthResponse() as Response);
    const client = createDesktopClient("external", false);

    render(<CollectorMonitor desktopClient={client} />);

    expect(await screen.findByText(/external collector/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /pause capture/i })).not.toBeInTheDocument();
    expect(client.stopCollector).not.toHaveBeenCalled();
  });
});
