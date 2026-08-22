import { FlaskConical, FolderOpen, ShieldCheck } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useScanStore } from "@/state/scanStore";
import { SettingsDialog } from "@/components/settings/SettingsDialog";

export function WelcomeView() {
  const mode = useScanStore((s) => s.mode);
  const setMode = useScanStore((s) => s.setMode);
  const chooseTargetDirectory = useScanStore((s) => s.chooseTargetDirectory);
  const playbookLoading = useScanStore((s) => s.playbookLoading);
  const playbookError = useScanStore((s) => s.playbookError);

  return (
    <div className="relative flex h-screen w-screen items-center justify-center bg-background px-6">
      <div className="absolute top-4 right-4">
        <SettingsDialog />
      </div>
      <div className="w-full max-w-md">
        <div className="mb-8 flex flex-col items-center text-center">
          <div className="mb-4 flex size-12 items-center justify-center rounded-xl bg-primary text-primary-foreground">
            <ShieldCheck className="size-6" />
          </div>
          <h1 className="text-lg font-semibold text-foreground">Vuln-Hound</h1>
          <p className="mt-1.5 text-sm text-muted-foreground">
            Point it at a codebase. It scans, explains, and fixes.
          </p>
        </div>

        <div className="rounded-xl border border-border bg-card p-5 shadow-sm">
          <div className="mb-4">
            <div className="mb-2 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
              Scan mode
            </div>
            <div className="flex rounded-md border border-border bg-muted p-0.5">
              <ModeOption
                active={mode === "test"}
                onClick={() => setMode("test")}
                icon={<FlaskConical className="size-3.5" />}
                label="Test"
              />
              <ModeOption
                active={mode === "production"}
                onClick={() => setMode("production")}
                icon={<ShieldCheck className="size-3.5" />}
                label="Production"
              />
            </div>
            <p className="mt-2 text-[12px] leading-snug text-muted-foreground">
              {mode === "test"
                ? "Full scanning, including active checks. Use on a local or staging copy."
                : "Read-only, static analysis only. No live requests, no writes to your target's data."}
            </p>
          </div>

          <Button
            className="w-full gap-2"
            size="lg"
            onClick={chooseTargetDirectory}
            disabled={playbookLoading}
          >
            <FolderOpen className="size-4" />
            Choose a folder to scan
          </Button>

          {playbookError && (
            <p className="mt-3 text-[12px] text-severity-critical">
              Couldn't load the checklist: {playbookError}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

function ModeOption({
  active,
  onClick,
  icon,
  label,
}: {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  label: string;
}) {
  return (
    <Button
      type="button"
      variant="ghost"
      size="sm"
      onClick={onClick}
      className={cn(
        "flex-1 gap-1.5 rounded-[6px] text-xs",
        active
          ? "bg-primary text-primary-foreground hover:bg-primary/90"
          : "text-muted-foreground hover:bg-transparent hover:text-foreground"
      )}
    >
      {icon}
      {label}
    </Button>
  );
}
