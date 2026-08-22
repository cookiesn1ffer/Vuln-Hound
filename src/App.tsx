import { useEffect } from "react";
import { WelcomeView } from "@/routes/WelcomeView";
import { ScanView } from "@/routes/ScanView";
import { useScanStore } from "@/state/scanStore";
import { Toaster } from "@/components/ui/sonner";

function App() {
  const targetDir = useScanStore((s) => s.targetDir);
  const loadPlaybook = useScanStore((s) => s.loadPlaybook);
  const subscribeToScanEvents = useScanStore((s) => s.subscribeToScanEvents);

  useEffect(() => {
    loadPlaybook();
    subscribeToScanEvents();
  }, [loadPlaybook, subscribeToScanEvents]);

  return (
    <>
      {targetDir ? <ScanView /> : <WelcomeView />}
      <Toaster />
    </>
  );
}

export default App;
