import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { MOCK_FINDINGS, MOCK_GROUP_STATES } from "@/lib/mockData";
import { fetchPlaybook, pickTargetDirectory, startScanSession } from "@/lib/tauriClient";
import type { ScanEventPayload } from "@/lib/scanEvents";
import type { Finding, GroupRunState, Playbook, ScanMode, Severity } from "@/lib/playbookTypes";

const EMPTY_PLAYBOOK: Playbook = { version: 0, groups: [] };
const ZERO_COUNTS: Record<Severity, number> = { low: 0, medium: 0, high: 0, critical: 0 };

interface ScanState {
  targetDir: string | null;
  mode: ScanMode;
  playbook: Playbook;
  playbookLoading: boolean;
  playbookError: string | null;
  groupStates: Record<string, GroupRunState>;
  groupActivity: Record<string, string>;
  findings: Finding[];
  selectedGroupId: string | null;
  selectedFindingId: string | null;
  eventsSubscribed: boolean;

  loadPlaybook: () => Promise<void>;
  chooseTargetDirectory: () => Promise<void>;
  setMode: (mode: ScanMode) => void;
  selectGroup: (groupId: string | null) => void;
  selectFinding: (findingId: string | null) => void;
  runGroupScan: (groupId: string) => Promise<void>;
  subscribeToScanEvents: () => Promise<void>;
  loadDevPreviewData: () => void;
}

export const useScanStore = create<ScanState>((set, get) => ({
  targetDir: null,
  mode: "test",
  playbook: EMPTY_PLAYBOOK,
  playbookLoading: false,
  playbookError: null,
  groupStates: {},
  groupActivity: {},
  findings: [],
  selectedGroupId: null,
  selectedFindingId: null,
  eventsSubscribed: false,

  loadPlaybook: async () => {
    set({ playbookLoading: true, playbookError: null });
    try {
      const playbook = await fetchPlaybook();
      const firstGroupId = playbook.groups.slice().sort((a, b) => a.order - b.order)[0]?.id ?? null;
      set({
        playbook,
        playbookLoading: false,
        selectedGroupId: get().selectedGroupId ?? firstGroupId,
      });
    } catch (err) {
      set({ playbookLoading: false, playbookError: String(err) });
    }
  },

  chooseTargetDirectory: async () => {
    const dir = await pickTargetDirectory();
    if (dir) {
      set({ targetDir: dir, groupStates: {}, findings: [], selectedFindingId: null });
    }
  },

  setMode: (mode) => set({ mode }),
  selectGroup: (groupId) => set({ selectedGroupId: groupId, selectedFindingId: null }),
  selectFinding: (findingId) => set({ selectedFindingId: findingId }),

  runGroupScan: async (groupId) => {
    const { targetDir, mode } = get();
    if (!targetDir) return;

    set((s) => ({
      groupStates: {
        ...s.groupStates,
        [groupId]: { groupId, status: "scanning", counts: { ...ZERO_COUNTS } },
      },
      groupActivity: { ...s.groupActivity, [groupId]: "Starting…" },
      findings: s.findings.filter((f) => f.groupId !== groupId),
    }));

    try {
      await startScanSession(targetDir, mode, groupId);
    } catch (err) {
      set((s) => ({
        groupStates: {
          ...s.groupStates,
          [groupId]: { groupId, status: "error", counts: { ...ZERO_COUNTS } },
        },
      }));
      toast.error(`Couldn't start scan: ${String(err)}`);
    }
  },

  subscribeToScanEvents: async () => {
    if (get().eventsSubscribed) return;
    set({ eventsSubscribed: true });

    await listen<ScanEventPayload>("scan_event", (event) => {
      const payload = event.payload;

      if (payload.type === "status") {
        set((s) => ({
          groupStates: {
            ...s.groupStates,
            [payload.groupId]: {
              groupId: payload.groupId,
              status: payload.status,
              counts: s.groupStates[payload.groupId]?.counts ?? { ...ZERO_COUNTS },
            },
          },
        }));
        return;
      }

      if (payload.type === "finding") {
        set((s) => {
          const prevCounts = s.groupStates[payload.groupId]?.counts ?? { ...ZERO_COUNTS };
          return {
            findings: [...s.findings, payload.finding],
            groupStates: {
              ...s.groupStates,
              [payload.groupId]: {
                groupId: payload.groupId,
                status: "scanning",
                counts: {
                  ...prevCounts,
                  [payload.finding.severity]: prevCounts[payload.finding.severity] + 1,
                },
              },
            },
          };
        });
        return;
      }

      if (payload.type === "log") {
        set((s) => ({ groupActivity: { ...s.groupActivity, [payload.groupId]: payload.message } }));
        return;
      }

      if (payload.type === "error") {
        toast.error(payload.message);
      }
    });
  },

  // Dev-only helper to preview the findings UI before real scanning existed.
  loadDevPreviewData: () => set({ groupStates: MOCK_GROUP_STATES, findings: MOCK_FINDINGS }),
}));
