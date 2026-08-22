import { CheckCircle2, Loader2, XCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import { SeverityBadge } from "./SeverityBadge";
import { SEVERITY_COLOR_VAR } from "@/lib/severity";
import type { Finding } from "@/lib/playbookTypes";

export function FindingRow({
  finding,
  active,
  onSelect,
}: {
  finding: Finding;
  active: boolean;
  onSelect: () => void;
}) {
  const primaryEvidence = finding.evidence[0];

  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        "flex w-full items-center gap-3 border-b border-border px-4 py-3 text-left transition-colors",
        active ? "bg-accent/40" : "hover:bg-muted/60"
      )}
    >
      <span
        className="h-8 w-[3px] shrink-0 rounded-full"
        style={{ backgroundColor: SEVERITY_COLOR_VAR[finding.severity] }}
      />

      <div className="min-w-0 flex-1">
        <div className="truncate text-[13px] font-medium text-foreground">{finding.title}</div>
        {primaryEvidence && (
          <div className="truncate font-mono text-[11px] text-muted-foreground">
            {primaryEvidence.filePath}
            {primaryEvidence.lineStart ? `:${primaryEvidence.lineStart}` : ""}
          </div>
        )}
      </div>

      <SeverityBadge severity={finding.severity} />
      <StatusChip status={finding.status} />
    </button>
  );
}

function StatusChip({ status }: { status: Finding["status"] }) {
  if (status === "fixing" || status === "reverifying") {
    return <Loader2 className="size-3.5 shrink-0 animate-spin text-accent" />;
  }
  if (status === "fixed") {
    return <CheckCircle2 className="size-3.5 shrink-0 text-severity-low" />;
  }
  if (status === "fix_failed") {
    return <XCircle className="size-3.5 shrink-0 text-severity-critical" />;
  }
  return null;
}
