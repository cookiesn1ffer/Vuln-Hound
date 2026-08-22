export type Severity = "low" | "medium" | "high" | "critical";

export type ScanMode = "test" | "production";

export type GroupStatus = "idle" | "scanning" | "scanned" | "error";

export type FindingStatus =
  | "pending"
  | "scanning"
  | "found"
  | "not_found"
  | "fixing"
  | "fixed"
  | "fix_failed"
  | "reverifying";

export interface PlaybookEntry {
  id: string;
  name: string;
  baseSeverity: Severity;
  whatItIs: string;
  howItHurtsTemplate: string;
  fixGuidanceTemplate: string;
  requiresDynamicTest: boolean;
}

export interface PlaybookGroup {
  id: string;
  order: number;
  title: string;
  description: string;
  entries: PlaybookEntry[];
}

export interface Playbook {
  version: number;
  groups: PlaybookGroup[];
}

export interface FindingEvidence {
  filePath: string;
  lineStart?: number;
  lineEnd?: number;
  snippet?: string;
}

export interface Finding {
  id: string;
  groupId: string;
  categoryId: string;
  title: string;
  severity: Severity;
  status: FindingStatus;
  description: string;
  evidence: FindingEvidence[];
  fixGuidance: string;
  fixDiff?: string;
  fixSummary?: string;
  reverifyResult?: "confirmed_fixed" | "still_present" | "inconclusive";
  createdAt: string;
  updatedAt: string;
}

export interface GroupRunState {
  groupId: string;
  status: GroupStatus;
  counts: Record<Severity, number>;
}
