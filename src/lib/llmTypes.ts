export interface ProviderPreset {
  id: string;
  label: string;
  family: string | null;
  baseUrl: string;
  requiresKey: boolean;
}

export interface Settings {
  providerId: string;
  baseUrl: string;
  model: string;
}
