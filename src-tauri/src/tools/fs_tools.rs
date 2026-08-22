use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use serde::Serialize;

use super::error::ToolError;

const ALWAYS_SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    "target",
    ".venv",
    "venv",
    "__pycache__",
    ".next",
    ".cache",
    ".turbo",
];

const MANIFEST_FILENAMES: &[&str] = &[
    "package.json",
    "requirements.txt",
    "Pipfile",
    "pyproject.toml",
    "go.mod",
    "Cargo.toml",
    "composer.json",
    "Gemfile",
    "pom.xml",
    "build.gradle",
];

const MAX_READ_BYTES: usize = 200_000;

/// Resolves `rel_path` against `root`, verifying the result stays inside `root`.
/// This is the same defense-in-depth principle the app teaches for path traversal
/// findings, applied to the tool that reads the target codebase itself.
fn resolve_within_root(root: &Path, rel_path: &str) -> Result<PathBuf, ToolError> {
    let candidate = root.join(rel_path);
    let canonical_root = root.canonicalize()?;
    let canonical_candidate = candidate
        .canonicalize()
        .map_err(|_| ToolError::NotFound(rel_path.to_string()))?;

    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(ToolError::PathEscape);
    }
    Ok(canonical_candidate)
}

fn walker(root: &Path) -> WalkBuilder {
    let mut builder = WalkBuilder::new(root);
    builder.hidden(false).git_ignore(true).git_global(true);
    builder.filter_entry(|entry| {
        let name = entry.file_name().to_string_lossy();
        !ALWAYS_SKIP_DIRS.contains(&name.as_ref())
    });
    builder
}

#[derive(Debug, Serialize)]
pub struct DirEntryInfo {
    pub path: String,
    #[serde(rename = "isDir")]
    pub is_dir: bool,
}

pub fn list_directory(root: &Path, max_entries: usize) -> Result<Vec<DirEntryInfo>, ToolError> {
    let mut out = Vec::new();
    for result in walker(root).build() {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.depth() == 0 {
            continue; // skip the root itself
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        out.push(DirEntryInfo { path: rel, is_dir });
        if out.len() >= max_entries {
            break;
        }
    }
    Ok(out)
}

pub fn read_file(root: &Path, rel_path: &str) -> Result<String, ToolError> {
    let full_path = resolve_within_root(root, rel_path)?;
    let bytes = std::fs::read(&full_path)?;
    let truncated = &bytes[..bytes.len().min(MAX_READ_BYTES)];
    String::from_utf8(truncated.to_vec()).map_err(|_| ToolError::NotUtf8)
}

#[derive(Debug, Serialize)]
pub struct SearchMatch {
    pub path: String,
    #[serde(rename = "lineNumber")]
    pub line_number: u64,
    pub line: String,
}

/// A [`grep_searcher::Sink`] that collects matches (up to a remaining budget)
/// into a plain `Vec`, tagging each with the relative file path being searched.
struct CollectSink<'a> {
    rel_path: &'a str,
    remaining: usize,
    out: Vec<SearchMatch>,
}

impl<'a> grep_searcher::Sink for CollectSink<'a> {
    type Error = std::io::Error;

    fn matched(
        &mut self,
        _searcher: &grep_searcher::Searcher,
        mat: &grep_searcher::SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        if self.out.len() >= self.remaining {
            return Ok(false);
        }
        let line = String::from_utf8_lossy(mat.bytes()).trim_end().to_string();
        self.out.push(SearchMatch {
            path: self.rel_path.to_string(),
            line_number: mat.line_number().unwrap_or(0),
            line,
        });
        Ok(self.out.len() < self.remaining)
    }
}

pub fn search_text(
    root: &Path,
    pattern: &str,
    max_matches: usize,
) -> Result<Vec<SearchMatch>, ToolError> {
    use grep_regex::RegexMatcher;
    use grep_searcher::SearcherBuilder;

    let matcher =
        RegexMatcher::new(pattern).map_err(|e| ToolError::InvalidPattern(e.to_string()))?;
    let mut searcher = SearcherBuilder::new().line_number(true).build();
    let mut out: Vec<SearchMatch> = Vec::new();

    for result in walker(root).build() {
        if out.len() >= max_matches {
            break;
        }
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.file_type().map(|t| !t.is_file()).unwrap_or(true) {
            continue;
        }
        let path = entry.path().to_path_buf();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let mut sink = CollectSink {
            rel_path: &rel,
            remaining: max_matches - out.len(),
            out: Vec::new(),
        };
        if searcher.search_path(&matcher, &path, &mut sink).is_ok() {
            out.extend(sink.out);
        }
    }

    Ok(out)
}

#[derive(Debug, Serialize)]
pub struct ManifestFile {
    pub path: String,
    pub content: String,
}

pub fn get_dependency_manifests(root: &Path) -> Result<Vec<ManifestFile>, ToolError> {
    let mut out = Vec::new();
    for result in walker(root).max_depth(Some(4)).build() {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.file_type().map(|t| !t.is_file()).unwrap_or(true) {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !MANIFEST_FILENAMES.contains(&file_name.as_str()) {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();
        if let Ok(content) = read_file(root, &rel) {
            out.push(ManifestFile { path: rel, content });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("package.json"), r#"{"name":"demo"}"#).unwrap();
        fs::write(
            dir.path().join("src/server.js"),
            "const key = \"sk_live_abc123\";\nconsole.log(key);\n",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("node_modules/pkg")).unwrap();
        fs::write(dir.path().join("node_modules/pkg/index.js"), "noop").unwrap();
        dir
    }

    #[test]
    fn list_directory_skips_node_modules() {
        let dir = setup();
        let entries = list_directory(dir.path(), 100).unwrap();
        assert!(entries.iter().any(|e| e.path == "package.json"));
        assert!(entries.iter().any(|e| e.path.contains("server.js")));
        assert!(!entries.iter().any(|e| e.path.contains("node_modules")));
    }

    #[test]
    fn read_file_returns_contents() {
        let dir = setup();
        let content = read_file(dir.path(), "src/server.js").unwrap();
        assert!(content.contains("sk_live_abc123"));
    }

    #[test]
    fn read_file_rejects_path_escape() {
        let dir = setup();
        let result = read_file(dir.path(), "../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn search_text_finds_pattern() {
        let dir = setup();
        let matches = search_text(dir.path(), "sk_live_[a-z0-9]+", 10).unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "src/server.js");
    }

    #[test]
    fn get_dependency_manifests_finds_package_json() {
        let dir = setup();
        let manifests = get_dependency_manifests(dir.path()).unwrap();
        assert!(manifests.iter().any(|m| m.path == "package.json"));
    }
}
