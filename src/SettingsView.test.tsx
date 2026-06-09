import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SettingsView } from "./SettingsView";
import type { DesktopConfigClient, DesktopConfigView } from "./lib/desktopConfig";

const baseView: DesktopConfigView = {
  configPath: "C:/Users/example/AppData/Roaming/tsr/config.json",
  firstRun: false,
  aiSecretStatus: {
    present: true,
    masked: "••••CRET"
  },
  config: {
    schemaVersion: 1,
    storage: {
      dataDir: "C:/Users/example/AppData/Local/tsr/data",
      databasePath: "C:/Users/example/AppData/Local/tsr/data/local.sqlite3",
      screenshotDir: "C:/Users/example/AppData/Local/tsr/data/screenshots",
      highResScreenshotDir:
        "C:/Users/example/AppData/Local/tsr/data/high-res-screenshots",
      retentionDays: 30
    },
    capture: {
      pollMs: 1000,
      screenshotIntervalSecs: 60,
      highResCaptureEnabled: true,
      inputCaptureEnabled: true,
      idleThresholdSecs: 120
    },
    privacy: {
      defaultPrivacyMode: "redacted",
      blockerConfigPath: "C:/Users/example/AppData/Local/tsr/data/blocker_config.json",
      externalAiWarningAccepted: false
    },
    ai: {
      enabled: false,
      providerPreset: "customOpenAiCompatible",
      displayName: "Custom OpenAI-compatible provider",
      baseUrl: "",
      model: "gpt-4o-mini",
      maxCompletionTokens: 200000,
      visionEnabled: true,
      pipelines: {
        visualAnalysis: false,
        insightReports: false,
        dailyBrief: false
      }
    },
    system: {
      apiPort: 4317,
      launchOnStartup: false,
      startMinimized: false,
      trayEnabled: true
    }
  }
};

function createClient(view = baseView): DesktopConfigClient {
  let currentView = structuredClone(view);
  return {
    getConfig: vi.fn(async () => currentView),
    saveConfig: vi.fn(async (config) => {
      currentView = { ...currentView, config };
      return currentView;
    }),
    setApiKey: vi.fn(async () => ({
      present: true,
      masked: "••••7890"
    })),
    clearApiKey: vi.fn(async () => ({
      present: false,
      masked: undefined
    })),
    chooseDataDirectory: vi.fn(async () => "D:/TimeRecorderData"),
    testProvider: vi.fn(async () => ({
      status: "ready",
      requestKind: "openai_compatible_text_chat_completions",
      endpoint: "https://api.minimax.example/v1/chat/completions",
      model: "MiniMax-M3",
      usesScreenshots: false,
      secret: {
        present: true,
        masked: "••••7890"
      },
      message: "Provider test is configured for a text-only chat completions request."
    }))
  };
}

describe("SettingsView", () => {
  it("loads storage and AI provider settings with masked key status", async () => {
    render(<SettingsView client={createClient()} />);

    expect(await screen.findByRole("heading", { name: /settings/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/data directory/i)).toHaveValue(
      "C:/Users/example/AppData/Local/tsr/data"
    );
    expect(screen.getByLabelText(/provider preset/i)).toHaveValue(
      "customOpenAiCompatible"
    );
    expect(screen.getByText("••••CRET")).toBeInTheDocument();
  });

  it("renders first-run setup and completes with local-only defaults", async () => {
    const client = createClient({ ...baseView, firstRun: true });
    render(<SettingsView client={client} />);

    expect(await screen.findByRole("heading", { name: /first-run setup/i })).toBeInTheDocument();
    expect(screen.getByText(/Local Only/i)).toBeInTheDocument();
    expect(screen.getByText(/External AI can send text summaries/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /complete setup/i }));
    await waitFor(() => expect(client.saveConfig).toHaveBeenCalledTimes(1));
    expect(vi.mocked(client.saveConfig).mock.calls[0][0].ai.enabled).toBe(false);
  });

  it("lets the user choose a data directory from the desktop picker", async () => {
    const client = createClient();
    render(<SettingsView client={client} />);

    fireEvent.click(await screen.findByRole("button", { name: /choose data directory/i }));

    expect(await screen.findByLabelText(/data directory/i)).toHaveValue("D:/TimeRecorderData");
    expect(screen.getByLabelText(/database path/i)).toHaveValue(
      "D:/TimeRecorderData/local.sqlite3"
    );
    expect(client.chooseDataDirectory).toHaveBeenCalledTimes(1);
  });

  it("saves provider settings without sending the API key through config save", async () => {
    const client = createClient();
    render(<SettingsView client={client} />);

    fireEvent.click(await screen.findByLabelText(/enable external ai/i));
    fireEvent.change(screen.getByLabelText(/provider preset/i), {
      target: { value: "miniMax" }
    });
    fireEvent.change(screen.getByLabelText(/base url/i), {
      target: { value: "https://api.minimax.example/v1" }
    });
    fireEvent.change(screen.getByLabelText(/^model$/i), {
      target: { value: "MiniMax-M3" }
    });
    fireEvent.click(screen.getByLabelText(/external ai warning accepted/i));
    fireEvent.change(screen.getByLabelText(/api key/i), {
      target: { value: "sk-live-secret-value-7890" }
    });

    fireEvent.click(screen.getByRole("button", { name: /save settings/i }));
    await waitFor(() => expect(client.saveConfig).toHaveBeenCalledTimes(1));

    const savedConfig = vi.mocked(client.saveConfig).mock.calls[0][0];
    expect(JSON.stringify(savedConfig)).not.toContain("sk-live-secret-value-7890");
    expect(savedConfig.ai.providerPreset).toBe("miniMax");

    fireEvent.click(screen.getByRole("button", { name: /save api key/i }));
    await waitFor(() => expect(client.setApiKey).toHaveBeenCalledWith("sk-live-secret-value-7890"));
    expect(await screen.findByText("••••7890")).toBeInTheDocument();
  });

  it("tests provider readiness through a text-only path", async () => {
    const client = createClient({
      ...baseView,
      config: {
        ...baseView.config,
        ai: {
          ...baseView.config.ai,
          enabled: true,
          providerPreset: "miniMax",
          baseUrl: "https://api.minimax.example/v1",
          model: "MiniMax-M3"
        }
      }
    });
    render(<SettingsView client={client} />);

    fireEvent.click(await screen.findByRole("button", { name: /test provider/i }));

    expect(await screen.findByText(/text_chat_completions/i)).toBeInTheDocument();
    expect(screen.getByText(/Screenshots: no/i)).toBeInTheDocument();
    expect(screen.queryByText(/sk-live-secret/i)).not.toBeInTheDocument();
  });
});
