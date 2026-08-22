import { AppShell } from "@/components/layout/AppShell";
import { Sidebar } from "@/components/layout/Sidebar";
import { FindingList } from "@/components/findings/FindingList";
import { FindingDetailPanel } from "@/components/detail/FindingDetailPanel";
import { useScanStore } from "@/state/scanStore";

export function ScanView() {
  const selectedFindingId = useScanStore((s) => s.selectedFindingId);

  return (
    <AppShell
      sidebar={<Sidebar />}
      main={<FindingList />}
      detail={selectedFindingId ? <FindingDetailPanel /> : null}
    />
  );
}
