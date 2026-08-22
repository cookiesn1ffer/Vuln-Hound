import type { Finding, GroupStatus } from "./playbookTypes";

export type ScanEventPayload =
  | { type: "status"; sessionId: string; groupId: string; status: GroupStatus }
  | { type: "finding"; sessionId: string; groupId: string; finding: Finding }
  | { type: "log"; sessionId: string; groupId: string; message: string }
  | { type: "error"; sessionId: string; groupId: string; message: string };
