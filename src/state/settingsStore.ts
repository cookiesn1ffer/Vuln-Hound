import { create } from "zustand";
import {
  deleteApiKey,
  fetchProviderPresets,
  fetchSettings,
  getApiKeyStatus,
  saveApiKey,
  saveSettings,
  testProviderConnection,
} from "@/lib/tauriClient";
import type { ProviderPreset, Settings } from "@/lib/llmTypes";

type TestResult = { ok: true; reply: string } | { ok: false; error: string } | null;

interface SettingsState {
  presets: ProviderPreset[];
  settings: Settings;
  apiKeyStatus: Record<string, boolean>;
  loaded: boolean;
  saving: boolean;
  testing: boolean;
  testResult: TestResult;

  load: () => Promise<void>;
  setProvider: (providerId: string) => void;
  setBaseUrl: (baseUrl: string) => void;
  setModel: (model: string) => void;
  save: () => Promise<void>;
  saveKey: (key: string) => Promise<void>;
  clearKey: () => Promise<void>;
  testConnection: () => Promise<void>;
}

const FALLBACK_SETTINGS: Settings = { providerId: "lmstudio", baseUrl: "http://localhost:1234/v1", model: "local-model" };

export const useSettingsStore = create<SettingsState>((set, get) => ({
  presets: [],
  settings: FALLBACK_SETTINGS,
  apiKeyStatus: {},
  loaded: false,
  saving: false,
  testing: false,
  testResult: null,

  load: async () => {
    const [presets, settings] = await Promise.all([fetchProviderPresets(), fetchSettings()]);
    const statusEntries = await Promise.all(
      presets.map(async (p) => [p.id, await getApiKeyStatus(p.id)] as const)
    );
    set({
      presets,
      settings,
      apiKeyStatus: Object.fromEntries(statusEntries),
      loaded: true,
    });
  },

  setProvider: (providerId) => {
    const preset = get().presets.find((p) => p.id === providerId);
    if (!preset) return;
    set({
      settings: {
        providerId: preset.id,
        baseUrl: preset.baseUrl,
        model: preset.defaultModel ?? "",
      },
      testResult: null,
    });
  },

  setBaseUrl: (baseUrl) => set((s) => ({ settings: { ...s.settings, baseUrl } })),
  setModel: (model) => set((s) => ({ settings: { ...s.settings, model } })),

  save: async () => {
    set({ saving: true });
    try {
      await saveSettings(get().settings);
    } finally {
      set({ saving: false });
    }
  },

  saveKey: async (key) => {
    const providerId = get().settings.providerId;
    await saveApiKey(providerId, key);
    set((s) => ({ apiKeyStatus: { ...s.apiKeyStatus, [providerId]: true } }));
  },

  clearKey: async () => {
    const providerId = get().settings.providerId;
    await deleteApiKey(providerId);
    set((s) => ({ apiKeyStatus: { ...s.apiKeyStatus, [providerId]: false } }));
  },

  testConnection: async () => {
    set({ testing: true, testResult: null });
    try {
      await get().save();
      const reply = await testProviderConnection();
      set({ testing: false, testResult: { ok: true, reply } });
    } catch (err) {
      set({ testing: false, testResult: { ok: false, error: String(err) } });
    }
  },
}));
