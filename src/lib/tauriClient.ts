import { invoke } from "@tauri-apps/api/core";
import type { Playbook, ScanMode } from "./playbookTypes";
import type { ProviderPreset, Settings } from "./llmTypes";

export async function pickTargetDirectory(): Promise<string | null> {
  return invoke<string | null>("pick_target_directory");
}

export async function fetchPlaybook(): Promise<Playbook> {
  return invoke<Playbook>("get_playbook");
}

export async function fetchSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function saveSettings(settings: Settings): Promise<void> {
  return invoke("set_settings", { settings });
}

export async function fetchProviderPresets(): Promise<ProviderPreset[]> {
  return invoke<ProviderPreset[]>("get_provider_presets");
}

export async function saveApiKey(providerId: string, key: string): Promise<void> {
  return invoke("save_api_key", { providerId, key });
}

export async function getApiKeyStatus(providerId: string): Promise<boolean> {
  return invoke<boolean>("get_api_key_status", { providerId });
}

export async function deleteApiKey(providerId: string): Promise<void> {
  return invoke("delete_api_key", { providerId });
}

export async function testProviderConnection(): Promise<string> {
  return invoke<string>("test_provider_connection");
}

export async function startScanSession(
  targetDir: string,
  mode: ScanMode,
  groupId: string
): Promise<string> {
  return invoke<string>("start_scan_session", { targetDir, mode, groupId });
}

export async function cancelSession(sessionId: string): Promise<void> {
  return invoke("cancel_session", { sessionId });
}
