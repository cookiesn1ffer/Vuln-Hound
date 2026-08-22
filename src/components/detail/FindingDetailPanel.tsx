import { CheckCircle2, Loader2, RotateCcw, Sparkles, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { SeverityBadge } from "@/components/findings/SeverityBadge";
import { DiffView } from "./DiffView";
import { useScanStore } from "@/state/scanStore";

export function FindingDetailPanel() {
  const findings = useScanStore((s) => s.findings);
  const selectedFindingId = useScanStore((s) => s.selectedFindingId);
  const selectFinding = useScanStore((s) => s.selectFinding);
  const finding = findings.find((f) => f.id === selectedFindingId);

  if (!finding) return null;

  const isFixing = finding.status === "fixing" || finding.status === "reverifying";
  const isFixed = finding.status === "fixed";

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-start justify-between gap-2 border-b border-border px-5 py-4">
        <div className="min-w-0">
          <SeverityBadge severity={finding.severity} />
          <h2 className="mt-2 text-[15px] leading-snug font-semibold text-foreground">
            {finding.title}
          </h2>
        </div>
        <Button variant="ghost" size="icon-sm" onClick={() => selectFinding(null)}>
          <X className="size-4" />
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto px-5 py-4">
        <Section label="How this hurts your app">
          <p className="text-[13px] leading-relaxed text-foreground/90">{finding.description}</p>
        </Section>

        {finding.evidence.length > 0 && (
          <Section label="Evidence">
            <div className="space-y-2">
              {finding.evidence.map((ev, i) => (
                <div key={i} className="rounded-md border border-border bg-muted/60 p-2.5">
                  <div className="mb-1 font-mono text-[11px] text-muted-foreground">
                    {ev.filePath}
                    {ev.lineStart ? `:${ev.lineStart}` : ""}
                  </div>
                  {ev.snippet && (
                    <pre className="overflow-x-auto font-mono text-[11.5px] text-foreground">
                      {ev.snippet}
                    </pre>
                  )}
                </div>
              ))}
            </div>
          </Section>
        )}

        <Section label="Recommended fix">
          <p className="text-[13px] leading-relaxed text-foreground/90">{finding.fixGuidance}</p>
        </Section>

        {finding.fixDiff && (
          <Section label={isFixed ? "Applied change" : "Proposed change"}>
            <DiffView diff={finding.fixDiff} />
            {finding.fixSummary && (
              <p className="mt-2 text-[12px] text-muted-foreground">{finding.fixSummary}</p>
            )}
          </Section>
        )}
      </div>

      <div className="border-t border-border px-5 py-4">
        {isFixed ? (
          <div className="flex items-center justify-between gap-2">
            <span className="flex items-center gap-1.5 text-[12px] font-medium text-severity-low">
              <CheckCircle2 className="size-3.5" />
              Fixed
              {finding.reverifyResult === "confirmed_fixed" && " · re-verified"}
            </span>
            <Button variant="outline" size="sm" className="gap-1.5">
              <RotateCcw className="size-3.5" />
              Revert
            </Button>
          </div>
        ) : (
          <Button className="w-full gap-1.5" disabled={isFixing}>
            {isFixing ? (
              <>
                <Loader2 className="size-3.5 animate-spin" />
                Applying fix…
              </>
            ) : (
              <>
                <Sparkles className="size-3.5" />
                Fix this automatically
              </>
            )}
          </Button>
        )}
      </div>
    </div>
  );
}

function Section({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="mb-5 border-b border-border pb-5 last:border-0 last:pb-0">
      <div className="mb-1.5 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
        {label}
      </div>
      {children}
    </div>
  );
}
