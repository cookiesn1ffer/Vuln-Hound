import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { SEVERITY_BG_CLASS, SEVERITY_LABEL } from "@/lib/severity";
import type { Severity } from "@/lib/playbookTypes";

export function SeverityBadge({ severity, className }: { severity: Severity; className?: string }) {
  return (
    <Badge variant="outline" className={cn("font-medium", SEVERITY_BG_CLASS[severity], className)}>
      {SEVERITY_LABEL[severity]}
    </Badge>
  );
}

export function SeverityDot({ severity, className }: { severity: Severity; className?: string }) {
  return (
    <span
      className={cn("inline-block size-1.5 rounded-full", className)}
      style={{ backgroundColor: `var(--severity-${severity})` }}
    />
  );
}
