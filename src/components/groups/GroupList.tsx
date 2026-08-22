import { useScanStore } from "@/state/scanStore";
import { GroupCard } from "./GroupCard";
import type { GroupRunState } from "@/lib/playbookTypes";

const EMPTY_STATE = (groupId: string): GroupRunState => ({
  groupId,
  status: "idle",
  counts: { low: 0, medium: 0, high: 0, critical: 0 },
});

export function GroupList() {
  const playbook = useScanStore((s) => s.playbook);
  const groupStates = useScanStore((s) => s.groupStates);
  const selectedGroupId = useScanStore((s) => s.selectedGroupId);
  const selectGroup = useScanStore((s) => s.selectGroup);
  const runGroupScan = useScanStore((s) => s.runGroupScan);

  return (
    <div className="flex flex-col gap-1">
      {playbook.groups
        .slice()
        .sort((a, b) => a.order - b.order)
        .map((group) => (
          <GroupCard
            key={group.id}
            group={group}
            state={groupStates[group.id] ?? EMPTY_STATE(group.id)}
            active={selectedGroupId === group.id}
            onSelect={() => selectGroup(group.id)}
            onTest={() => runGroupScan(group.id)}
          />
        ))}
    </div>
  );
}
