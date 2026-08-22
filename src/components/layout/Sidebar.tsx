import { FlaskConical, FolderOpen, ShieldCheck } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useScanStore } from "@/state/scanStore";
import { GroupList } from "@/components/groups/GroupList";
import { SettingsDialog } from "@/components/settings/SettingsDialog";

export function Sidebar() {
  const targetDir = useScanStore((s) => s.targetDir);
  const mode = useScanStore((s) => s.mode);
  const setMode = useScanStore((s) => s.setMode);
  const chooseTargetDirectory = useScanStore((s) => s.chooseTargetDirectory);

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 border-b border-border px-4 py-4">
        <div className="flex size-7 items-center justify-center rounded-md bg-primary text-primary-foreground">
          <ShieldCheck className="size-4" />
        </div>
        <span className="text-sm font-semibold tracking-tight">Vuln-Hound</span>
        <div className="ml-auto">
          <SettingsDialog />
        </div>
      </div>

      <div className="border-b border-border px-4 py-3">
        <div className="mb-1.5 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
          Target
        </div>
        <button
          type="button"
          onClick={chooseTargetDirectory}
          className="flex w-full items-center gap-2 rounded-md border border-border bg-card px-2.5 py-2 text-left text-xs hover:bg-muted"
        >
          <FolderOpen className="size-3.5 shrink-0 text-muted-foreground" />
          <span className="truncate font-mono text-[11px] text-foreground">
            {targetDir ?? "Choose a folder…"}
          </span>
        </button>

        <div className="mt-3 flex rounded-md border border-border bg-card p-0.5">
          <ModeButton
            active={mode === "test"}
            onClick={() => setMode("test")}
            icon={<FlaskConical className="size-3.5" />}
            label="Test"
          />
          <ModeButton
            active={mode === "production"}
            onClick={() => setMode("production")}
            icon={<ShieldCheck className="size-3.5" />}
            label="Production"
          />
        </div>
        {mode === "production" && (
          <p className="mt-1.5 text-[11px] leading-snug text-muted-foreground">
            Read-only scanning. No live requests, no data writes to your target.
          </p>
        )}
      </div>

      <div className="flex-1 overflow-y-auto px-2 py-3">
        <div className="mb-1.5 px-2 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
          Checklist
        </div>
        <GroupList />
      </div>
    </div>
  );
}

function ModeButton({
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
