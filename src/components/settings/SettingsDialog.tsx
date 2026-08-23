import { useEffect, useMemo, useState } from "react";
import {
  CheckCircle2,
  Key,
  Loader2,
  RefreshCw,
  Settings as SettingsIcon,
  XCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useSettingsStore } from "@/state/settingsStore";
import type { ProviderPreset } from "@/lib/llmTypes";

export function SettingsDialog() {
  const [open, setOpen] = useState(false);
  const [keyDraft, setKeyDraft] = useState("");
  const loaded = useSettingsStore((s) => s.loaded);
  const load = useSettingsStore((s) => s.load);
  const presets = useSettingsStore((s) => s.presets);
  const settings = useSettingsStore((s) => s.settings);
  const apiKeyStatus = useSettingsStore((s) => s.apiKeyStatus);
  const setProvider = useSettingsStore((s) => s.setProvider);
  const setBaseUrl = useSettingsStore((s) => s.setBaseUrl);
  const setModel = useSettingsStore((s) => s.setModel);
  const saveKey = useSettingsStore((s) => s.saveKey);
  const clearKey = useSettingsStore((s) => s.clearKey);
  const testing = useSettingsStore((s) => s.testing);
  const testResult = useSettingsStore((s) => s.testResult);
  const testConnection = useSettingsStore((s) => s.testConnection);
  const models = useSettingsStore((s) => s.models);
  const modelsLoading = useSettingsStore((s) => s.modelsLoading);
  const modelsError = useSettingsStore((s) => s.modelsError);
  const refreshModels = useSettingsStore((s) => s.refreshModels);

  useEffect(() => {
    if (open && !loaded) load();
  }, [open, loaded, load]);

  const preset = presets.find((p) => p.id === settings.providerId);
  const hasKey = apiKeyStatus[settings.providerId] ?? false;

  const { grouped, standalone } = useMemo(() => {
    const grouped = new Map<string, ProviderPreset[]>();
    const standalone: ProviderPreset[] = [];
    for (const p of presets) {
      if (p.family) {
        const list = grouped.get(p.family) ?? [];
        list.push(p);
        grouped.set(p.family, list);
      } else {
        standalone.push(p);
      }
    }
    return { grouped, standalone };
  }, [presets]);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger
        render={
          <Button variant="ghost" size="icon-sm" aria-label="Settings">
            <SettingsIcon className="size-4" />
          </Button>
        }
      />
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>LLM provider</DialogTitle>
          <DialogDescription>
            Vuln-Hound uses this model to scan and fix your code. Pick a free provider — local or
            cloud — and it's ready to go.
          </DialogDescription>
        </DialogHeader>

        {!loaded ? (
          <div className="flex items-center justify-center py-8 text-muted-foreground">
            <Loader2 className="size-5 animate-spin" />
          </div>
        ) : (
          <div className="flex flex-col gap-4">
            <div>
              <Label className="mb-1.5 text-xs text-muted-foreground">Provider</Label>
              <Select
                value={settings.providerId}
                onValueChange={(value) => value && setProvider(value)}
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Choose a provider" />
                </SelectTrigger>
                <SelectContent>
                  {standalone.map((p) => (
                    <SelectItem key={p.id} value={p.id}>
                      {p.label}
                    </SelectItem>
                  ))}
                  {Array.from(grouped.entries()).map(([family, items]) => (
                    <SelectGroup key={family}>
                      <SelectLabel>{family}</SelectLabel>
                      {items.map((p) => (
                        <SelectItem key={p.id} value={p.id}>
                          {p.label}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label className="mb-1.5 text-xs text-muted-foreground">Base URL</Label>
              <Input
                value={settings.baseUrl}
                onChange={(e) => setBaseUrl(e.target.value)}
                onBlur={() => refreshModels()}
                className="font-mono text-xs"
                placeholder="http://localhost:1234/v1"
              />
            </div>

            <div>
              <Label className="mb-1.5 flex items-center text-xs text-muted-foreground">
                Model
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="ml-auto"
                  onClick={() => refreshModels()}
                  disabled={modelsLoading}
                  aria-label="Refresh model list"
                >
                  <RefreshCw className={modelsLoading ? "size-3 animate-spin" : "size-3"} />
                </Button>
              </Label>

              {models.length > 0 ? (
                <Select value={settings.model} onValueChange={(v) => v && setModel(v)}>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Choose a model" />
                  </SelectTrigger>
                  <SelectContent>
                    {models.map((m) => (
                      <SelectItem key={m} value={m}>
                        {m}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              ) : (
                <Input
                  value={settings.model}
                  onChange={(e) => setModel(e.target.value)}
                  className="font-mono text-xs"
                  placeholder={modelsLoading ? "Loading models…" : "model name"}
                  disabled={modelsLoading}
                />
              )}

              {modelsError && (
                <p className="mt-1.5 text-[11px] text-severity-critical">
                  Couldn't list models: {modelsError}
                </p>
              )}
              {!modelsError && models.length === 0 && !modelsLoading && (
                <p className="mt-1.5 text-[11px] text-muted-foreground">
                  {preset?.requiresKey && !hasKey
                    ? "Save an API key below, then refresh to see available models."
                    : "No models found — type one manually, or refresh once the server is reachable."}
                </p>
              )}
            </div>

            {preset?.requiresKey && (
              <div>
                <Label className="mb-1.5 flex items-center gap-1.5 text-xs text-muted-foreground">
                  <Key className="size-3" />
                  API key
                  {hasKey && (
                    <span className="ml-auto flex items-center gap-1 text-severity-low">
                      <CheckCircle2 className="size-3" />
                      Saved
                    </span>
                  )}
                </Label>
                <div className="flex gap-2">
                  <Input
                    type="password"
                    value={keyDraft}
                    onChange={(e) => setKeyDraft(e.target.value)}
                    placeholder={hasKey ? "•••••••••••••••• (replace)" : "Paste your API key"}
                    className="font-mono text-xs"
                  />
                  <Button
                    variant="secondary"
                    size="sm"
                    disabled={!keyDraft}
                    onClick={async () => {
                      await saveKey(keyDraft);
                      setKeyDraft("");
                    }}
                  >
                    Save
                  </Button>
                  {hasKey && (
                    <Button variant="ghost" size="sm" onClick={clearKey}>
                      Clear
                    </Button>
                  )}
                </div>
              </div>
            )}

            {testResult && (
              <div
                className={
                  testResult.ok
                    ? "flex items-start gap-1.5 rounded-md bg-severity-low/10 p-2.5 text-[12px] text-foreground"
                    : "flex items-start gap-1.5 rounded-md bg-severity-critical/10 p-2.5 text-[12px] text-foreground"
                }
              >
                {testResult.ok ? (
                  <CheckCircle2 className="mt-0.5 size-3.5 shrink-0 text-severity-low" />
                ) : (
                  <XCircle className="mt-0.5 size-3.5 shrink-0 text-severity-critical" />
                )}
                <span className="break-all">
                  {testResult.ok ? `Connected — model replied: "${testResult.reply}"` : testResult.error}
                </span>
              </div>
            )}
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={testConnection} disabled={!loaded || testing} className="gap-1.5">
            {testing && <Loader2 className="size-3.5 animate-spin" />}
            Test connection
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
