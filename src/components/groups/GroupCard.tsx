import { AlertCircle, CheckCircle2, ChevronRight, Loader2, PlayCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { SeverityDot } from "@/components/findings/SeverityBadge";
import { SEVERITY_ORDER } from "@/lib/severity";
import type { GroupRunState, PlaybookGroup } from "@/lib/playbookTypes";

export function GroupCard({
  group,
  state,
  active,
  onSelect,
  onTest,
}: {
  group: PlaybookGroup;
  state: GroupRunState;
  active: boolean;
  onSelect: () => void;
  onTest: () => void;
}) {
  const totalFindings = Object.values(state.counts).reduce((a, b) => a + b, 0);

  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        "group w-full rounded-lg border px-2.5 py-2.5 text-left transition-colors",
        active
          ? "border-accent-soft bg-accent/40"
          : "border-transparent hover:border-border hover:bg-card"
      )}
    >
      <div className="flex items-center justify-between gap-2">
        <div className="flex min-w-0 items-center gap-1.5">
          <StatusIcon status={state.status} />
          <span className="truncate text-[13px] font-medium text-foreground">{group.title}</span>
        </div>
        <ChevronRight className="size-3.5 shrink-0 text-muted-foreground/60 group-hover:text-muted-foreground" />
      </div>

      <div className="mt-2 flex items-center justify-between gap-2">
        <div className="flex items-center gap-1">
          {totalFindings === 0 ? (
            <span
              className={cn(
                "text-[11px]",
                state.status === "error" ? "text-severity-critical" : "text-muted-foreground"
              )}
            >
              {state.status === "scanning"
                ? "Scanning…"
                : state.status === "idle"
                  ? "Not tested"
                  : state.status === "error"
                    ? "Scan failed"
                    : "No issues"}
            </span>
          ) : (
            SEVERITY_ORDER.filter((sev) => state.counts[sev] > 0).map((sev) => (
              <span key={sev} className="flex items-center gap-0.5 text-[11px] text-muted-foreground">
                <SeverityDot severity={sev} />
                {state.counts[sev]}
              </span>
            ))
          )}
        </div>

        <Button
          type="button"
          size="xs"
          variant={state.status === "idle" ? "secondary" : "ghost"}
          onClick={(e) => {
            e.stopPropagation();
            onTest();
          }}
          disabled={state.status === "scanning"}
          className="gap-1 text-[11px]"
        >
          <PlayCircle className="size-3" />
          {state.status === "idle" ? "Test" : "Re-test"}
        </Button>
      </div>
    </button>
  );
}

function StatusIcon({ status }: { status: GroupRunState["status"] }) {
  if (status === "scanning") return <Loader2 className="size-3.5 shrink-0 animate-spin text-accent" />;
  if (status === "scanned") return <CheckCircle2 className="size-3.5 shrink-0 text-muted-foreground" />;
  if (status === "error") return <AlertCircle className="size-3.5 shrink-0 text-severity-critical" />;
  return <span className="size-3.5 shrink-0 rounded-full border border-border" />;
}
