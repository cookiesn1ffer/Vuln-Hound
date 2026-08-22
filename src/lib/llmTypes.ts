export interface ProviderPreset {
  id: string;
  label: string;
  baseUrl: string;
  requiresKey: boolean;
  defaultModel: string | null;
}

export interface Settings {
  providerId: string;
  baseUrl: string;
  model: string;
}
