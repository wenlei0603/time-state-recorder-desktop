import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export type CollectorDesktopStatus = {
  status: string;
  managed: boolean;
  pid?: number | null;
  apiUrl: string;
  dataDir?: string | null;
  lastError?: string | null;
};

export type DesktopRuntimeEvent = "open-settings" | "open-daily-brief";

export type DesktopRuntimeClient = {
  getCollectorStatus: () => Promise<CollectorDesktopStatus>;
  startCollector: () => Promise<CollectorDesktopStatus>;
  stopCollector: () => Promise<CollectorDesktopStatus>;
  listenRuntimeEvent: (
    event: DesktopRuntimeEvent,
    handler: () => void
  ) => Promise<UnlistenFn>;
};

function hasTauriRuntime(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof (window as Window & { __TAURI_INTERNALS__?: unknown })
      .__TAURI_INTERNALS__ === "object"
  );
}

function invokeDesktop<T>(command: string): Promise<T> {
  if (!hasTauriRuntime()) {
    return Promise.reject(
      new Error("Desktop runtime controls are unavailable in browser preview.")
    );
  }
  return invoke<T>(command);
}

function listenDesktop(event: DesktopRuntimeEvent, handler: () => void): Promise<UnlistenFn> {
  if (!hasTauriRuntime()) {
    return Promise.resolve(() => undefined);
  }
  return listen(`tsr://${event}`, () => handler());
}

export const tauriDesktopRuntimeClient: DesktopRuntimeClient = {
  getCollectorStatus: () => invokeDesktop<CollectorDesktopStatus>("collector_status"),
  startCollector: () => invokeDesktop<CollectorDesktopStatus>("start_collector"),
  stopCollector: () => invokeDesktop<CollectorDesktopStatus>("stop_collector"),
  listenRuntimeEvent: listenDesktop
};
