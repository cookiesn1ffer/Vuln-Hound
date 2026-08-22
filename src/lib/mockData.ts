import type { Finding, GroupRunState } from "./playbookTypes";

// Sample findings used only for design/dev preview (see scanStore's devPreview action).
// Not part of the real app's default state — a freshly launched app starts empty
// until a real scan has run against the user's chosen target directory.

const now = new Date().toISOString();

export const MOCK_FINDINGS: Finding[] = [
  {
    id: "f1",
    groupId: "secrets-config",
    categoryId: "hardcoded-secrets",
    title: "Stripe secret key hardcoded in server.js",
    severity: "critical",
    status: "found",
    description:
      "The Stripe secret key is committed directly in server.js. Anyone with read access to this repository (or a leaked build artifact) can use it to charge cards, issue refunds, or pull customer payment data under your account.",
    evidence: [{ filePath: "server.js", lineStart: 12, lineEnd: 12, snippet: 'const stripe = require("stripe")("sk_live_51H...");' }],
    fixGuidance: "Move the key to an environment variable loaded at runtime, rotate the exposed key immediately, and add server.js's secret pattern to a pre-commit secret scanner.",
    createdAt: now,
    updatedAt: now,
  },
  {
    id: "f2",
    groupId: "secrets-config",
    categoryId: "verbose-errors",
    title: "Stack traces returned to the client on 500 errors",
    severity: "low",
    status: "found",
    description:
      "Unhandled exceptions return the raw Node.js stack trace in the HTTP response, revealing file paths, package versions, and internal logic to anyone who can trigger an error.",
    evidence: [{ filePath: "middleware/errorHandler.js", lineStart: 8, snippet: "res.status(500).json({ error: err.stack });" }],
    fixGuidance: "Return a generic error message to clients and log the full stack trace server-side only.",
    createdAt: now,
    updatedAt: now,
  },
  {
    id: "f3",
    groupId: "secrets-config",
    categoryId: "default-db-creds",
    title: "Postgres connection uses default 'postgres/postgres' credentials",
    severity: "critical",
    status: "fixed",
    description:
      "The database connection string uses the default username and password. If the database port is ever exposed, this is a one-line takeover of all application data.",
    evidence: [{ filePath: "db.js", lineStart: 4 }],
    fixGuidance: "Generate a strong unique password, store it via environment variable / secrets manager, and rotate immediately.",
    fixSummary: "Replaced hardcoded credentials with process.env.DATABASE_URL and added a .env.example placeholder.",
    fixDiff:
      "-const client = new Client({ user: 'postgres', password: 'postgres' });\n+const client = new Client({ connectionString: process.env.DATABASE_URL });",
    reverifyResult: "confirmed_fixed",
    createdAt: now,
    updatedAt: now,
  },
  {
    id: "f4",
    groupId: "injection-execution",
    categoryId: "sql-injection",
    title: "User search endpoint builds SQL via string concatenation",
    severity: "critical",
    status: "found",
    description:
      "The /api/search endpoint concatenates the raw query parameter into a SQL statement. An attacker can inject arbitrary SQL to read, modify, or delete any row in the database, including other users' data.",
    evidence: [{ filePath: "routes/search.js", lineStart: 22, snippet: 'db.query("SELECT * FROM users WHERE name = \'" + req.query.q + "\'")' }],
    fixGuidance: "Use parameterized queries / a query builder instead of string concatenation.",
    createdAt: now,
    updatedAt: now,
  },
  {
    id: "f5",
    groupId: "injection-execution",
    categoryId: "path-traversal",
    title: "File download endpoint doesn't validate the requested path",
    severity: "high",
    status: "found",
    description:
      "The /files/:name endpoint passes the filename straight to the filesystem. A request like ../../.env can read arbitrary files on the server.",
    evidence: [{ filePath: "routes/files.js", lineStart: 15 }],
    fixGuidance: "Resolve the path and verify it stays within the intended directory before reading the file.",
    createdAt: now,
    updatedAt: now,
  },
];

export const MOCK_GROUP_STATES: Record<string, GroupRunState> = {
  "secrets-config": {
    groupId: "secrets-config",
    status: "scanned",
    counts: { low: 1, medium: 0, high: 0, critical: 2 },
  },
  "injection-execution": {
    groupId: "injection-execution",
    status: "scanned",
    counts: { low: 0, medium: 0, high: 1, critical: 1 },
  },
  "auth-session": {
    groupId: "auth-session",
    status: "scanning",
    counts: { low: 0, medium: 0, high: 0, critical: 0 },
  },
};
