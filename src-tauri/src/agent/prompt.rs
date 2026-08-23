use crate::findings::Finding;
use crate::playbook::PlaybookGroup;
use crate::security::mode::ScanMode;

const AGENTIC_PREAMBLE: &str = "You are Vuln-Hound, a security code-review agent. You investigate a codebase using the tools provided and report real, evidence-backed findings. Only report a finding if you have actually located it in the code via a tool call — cite the exact file and, where possible, line numbers. Do not guess or report generic/hypothetical issues. If a category from the checklist below genuinely doesn't apply to this codebase, simply don't report it.\n\nInvestigation strategy — follow this order:\n1. Call list_directory first (no arguments) to see the project's actual file layout and languages before searching for anything.\n2. If the project is small (roughly under ~15 source files, as shown by list_directory), prefer just calling read_file on each source file directly and reading it carefully — this is far more reliable than guessing regex patterns, since a single wrong guess (e.g. searching for template-literal syntax when the code actually uses string concatenation with `+`) makes search_text report nothing even though the real issue is sitting right there in the file.\n3. For larger projects where reading everything isn't practical, use search_text to find candidate locations — but if two or three different patterns for the same checklist item come back empty, don't conclude the issue is absent; read the most relevant file(s) directly instead of continuing to guess patterns.\n4. Call read_file on any promising match to confirm the issue and get exact line numbers/snippet before reporting it. Never repeat the exact same search_text call twice.\n5. Once you've covered the checklist items that are plausible for this codebase (you do not need to exhaustively search every possible pattern), call submit_findings exactly once. Do not keep investigating indefinitely — a handful of well-chosen tool calls per item is enough.";

pub fn build_scan_system_prompt(group: &PlaybookGroup, mode: ScanMode) -> String {
    let mode_banner = match mode {
        ScanMode::Test => {
            "MODE: Test. This is a non-production target — static code review only in this build (no live network requests are available as tools yet)."
        }
        ScanMode::Production => {
            "MODE: Production. This is a live production target. You may ONLY perform static/read-only code review. Never assume any dynamic/active check was run — for checklist items that need live testing to confirm (e.g. rate limiting, brute-force protection), review the code for the relevant safeguard (e.g. rate-limit middleware) and note if it's inconclusive from static review alone."
        }
    };

    let checklist = group
        .entries
        .iter()
        .map(|e| {
            format!(
                "- [{id}] {name} (typical severity: {sev:?}): {what}",
                id = e.id,
                name = e.name,
                sev = e.base_severity,
                what = e.what_it_is
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "{preamble}\n\n{mode_banner}\n\nYou are investigating the category group \"{title}\" ({desc}). Checklist for this group:\n{checklist}\n\nWhen you have finished investigating every applicable item, call submit_findings exactly once with everything you found. If you find nothing in this group, call submit_findings with an empty array.",
        preamble = AGENTIC_PREAMBLE,
        mode_banner = mode_banner,
        title = group.title,
        desc = group.description,
        checklist = checklist,
    )
}

pub fn build_fix_system_prompt(finding: &Finding) -> String {
    format!(
        "You are Vuln-Hound, a security code-fix agent. Fix exactly one confirmed finding by editing the affected file(s).\n\nFinding: {title}\nSeverity: {severity:?}\nWhy it matters: {description}\nRecommended fix: {fix_guidance}\nEvidence: {evidence:?}\n\nRead the affected file(s) with read_file first to see their exact current contents, then call write_file_staged with the complete corrected file contents for each file you change. Make the smallest correct change that fully resolves the finding — do not refactor unrelated code. When done, call submit_fix_result exactly once.",
        title = finding.title,
        severity = finding.severity,
        description = finding.description,
        fix_guidance = finding.fix_guidance,
        evidence = finding.evidence,
    )
}
