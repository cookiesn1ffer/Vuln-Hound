# Vuln-Hound

An agentic security scanner for AI-generated ("vibe coded") apps. Point it at a local codebase, pick a category, and a real LLM — with tool access to read, search, and reason over your files — investigates and reports findings in plain language, with severity ratings and fix guidance. No hardcoded regex detectors: the LLM does the reasoning, using an OpenAI-compatible tool-calling loop against a checklist of 56 vulnerability categories.

Desktop app, built with **Tauri v2 + React + TypeScript**. Works with a local LLM (LM Studio, Ollama) or a free-tier cloud provider (Groq, OpenRouter, Gemini, OpenCode Zen/Go) — nothing hardcoded, model lists are fetched live from whichever provider you pick.

## Why

Most static analyzers are pattern-matchers: fast, but shallow, and blind to anything outside their ruleset. Vuln-Hound instead gives an LLM read-only tools (`list_directory`, `read_file`, `search_text`, `get_dependency_manifests`) and a scoped playbook, and lets it actually investigate the code the way a reviewer would — then requires it to submit structured findings (not free-text) via a synthetic `submit_findings` tool call.

## Checklist coverage

56 checks across 9 groups (`src-tauri/resources/playbook.json`), each independently triggerable from the UI:

1. Injection & Code Execution — SQLi, command injection, insecure deserialization, path traversal, SSRF, template injection
2. Authentication & Session Security — weak/no password hashing, session fixation, predictable tokens, broken logout, insecure reset flows
3. Authorization & Access Control — IDOR, missing row-level security, mass assignment, function-level authz
4. API & Network Security — CORS misconfig, rate limiting, HTTP method tampering, open redirects, unencrypted transit
5. Client-Side & Web App Security — stored/reflected/DOM XSS, missing CSP, clickjacking, exposed client-side keys
6. Secrets & Configuration — hardcoded secrets, default credentials, verbose error disclosure, weak crypto
7. Data & Storage — public storage buckets, unvalidated file uploads, race conditions
8. Dependency & Supply Chain — vulnerable/hallucinated dependencies, typosquatting
9. AI / LLM-Specific — prompt injection, insecure output handling, training data leakage

## Test mode vs. Production mode

Every scan runs in one of two modes, enforced at the tool-registry level (not just a prompt instruction — the model is never given a tool it isn't allowed to use):

- **Test mode**: full read-only static analysis plus scoped dynamic checks against the target.
- **Production mode**: strictly passive/read-only. No network calls, no mutating tools — safe to point at a live app.

`src-tauri/src/agent/tool_registry.rs` has unit tests asserting Production's registry can never contain a dynamic/network tool.

## Getting started

**Prerequisites**: Node.js, Rust (`rustup`), and the [Tauri v2 system dependencies](https://v2.tauri.app/start/prerequisites/) for your OS.

```bash
npm install
npm run tauri dev
```

Then in Settings, pick a provider:

- **Local, no API key**: LM Studio (start its local server) or Ollama — either one, running an OpenAI-compatible tool-calling model.
- **Cloud, free tier**: Groq, OpenRouter, Gemini, or OpenCode Zen/Go — paste an API key, model list loads live.

Pick a target folder, click **Test** on any category group, and findings stream in as the agent investigates.

## Project layout

```
src/                      React frontend (Vite + TypeScript)
  components/              UI: group list, finding list/detail, settings, diff view
  state/                   Zustand stores (scanStore, settingsStore)
  lib/                     Tauri IPC client, shared types

src-tauri/src/
  agent/                   The tool-use loop (loop_.rs), system prompt, event types
  llm/                     Provider-agnostic OpenAI-compatible client + model listing
  tools/                   Filesystem tool executors (list/read/search/manifests)
  security/                Scan-mode enforcement
  commands/                Tauri IPC command surface
  secrets.rs               OS-keychain-backed API key storage (never persisted to disk)

src-tauri/resources/playbook.json   The 56-entry vulnerability checklist
fixtures/vulnerable-sample-app/     Seeded-vulnerability app used for live integration testing
```

## Secrets

Runtime API keys entered in Settings are stored in the OS-native credential store (Keychain / Credential Manager / Secret Service) via the `keyring` crate — never written to disk in plaintext, never returned to the frontend.

The only place secrets live on disk is `.env` (gitignored), and only for the opt-in live integration tests below. Copy `.env.example` to `.env` and fill it in.

## Testing

```bash
cargo test                                          # unit tests (no network)
npx tsc --noEmit                                    # frontend typecheck

# Opt-in live integration tests — require a real provider (local or cloud)
# and .env configured (see .env.example):
cargo test --test live_scan -- --ignored --nocapture
cargo test --test live_models -- --ignored --nocapture
```

## Status

Core scan pipeline (M0–M4) is built and live-validated against real providers. Fix pipeline (agent edits code, shows a diff, re-verifies) is designed but not yet wired up.
