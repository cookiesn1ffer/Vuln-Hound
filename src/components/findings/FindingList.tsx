import { AlertCircle, ShieldQuestion } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useScanStore } from "@/state/scanStore";
import { sortBySeverityAsc } from "@/lib/severity";
import { FindingRow } from "./FindingRow";

export function FindingList() {
  const playbook = useScanStore((s) => s.playbook);
  const selectedGroupId = useScanStore((s) => s.selectedGroupId);
  const findings = useScanStore((s) => s.findings);
  const selectedFindingId = useScanStore((s) => s.selectedFindingId);
  const selectFinding = useScanStore((s) => s.selectFinding);
  const runGroupScan = useScanStore((s) => s.runGroupScan);
  const groupState = useScanStore((s) =>
    selectedGroupId ? s.groupStates[selectedGroupId] : undefined
  );
  const activity = useScanStore((s) => (selectedGroupId ? s.groupActivity[selectedGroupId] : undefined));
  const error = useScanStore((s) => (selectedGroupId ? s.groupErrors[selectedGroupId] : undefined));

  const group = playbook.groups.find((g) => g.id === selectedGroupId);

  if (!group) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2 text-muted-foreground">
        <ShieldQuestion className="size-8" />
        <p className="text-sm">Select a category from the sidebar to view findings.</p>
      </div>
    );
  }

  const groupFindings = sortBySeverityAsc(findings.filter((f) => f.groupId === group.id));

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-border px-6 py-5">
        <h1 className="text-base font-semibold text-foreground">{group.title}</h1>
        <p className="mt-1 text-sm text-muted-foreground">{group.description}</p>
      </div>

      <div className="flex-1 overflow-y-auto">
        {groupState?.status === "idle" && (
          <div className="flex flex-col items-center justify-center gap-2 px-6 py-16 text-center text-muted-foreground">
            <p className="text-sm">This category hasn't been tested yet.</p>
            <p className="text-xs">Click Test in the sidebar to scan for these issues.</p>
          </div>
        )}

        {groupState?.status === "scanning" && (
          <div className="flex flex-col items-center justify-center gap-2 px-6 py-16 text-center text-muted-foreground">
            <p className="text-sm">Scanning {group.title.toLowerCase()}…</p>
            {activity && <p className="max-w-md truncate font-mono text-[11px]">{activity}</p>}
          </div>
        )}

        {groupState?.status === "scanned" && groupFindings.length === 0 && (
          <div className="flex flex-col items-center justify-center gap-2 px-6 py-16 text-center text-muted-foreground">
            <p className="text-sm">No issues found in this category.</p>
          </div>
        )}

        {groupState?.status === "error" && (
          <div className="flex flex-col items-center justify-center gap-3 px-6 py-16 text-center">
            <AlertCircle className="size-6 text-severity-critical" />
            <p className="text-sm text-foreground">The scan failed before finishing.</p>
            {error && <p className="max-w-md text-xs text-muted-foreground break-words">{error}</p>}
            <Button size="sm" variant="outline" onClick={() => selectedGroupId && runGroupScan(selectedGroupId)}>
              Retry
            </Button>
          </div>
        )}

        {groupFindings.map((finding) => (
          <FindingRow
            key={finding.id}
            finding={finding}
            active={finding.id === selectedFindingId}
            onSelect={() => selectFinding(finding.id)}
          />
        ))}
      </div>
    </div>
  );
}
