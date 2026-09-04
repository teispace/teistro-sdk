//! Repository tasks, in Rust, so contributors and CI need one toolchain.
//!
//! Invoked as `cargo xtask <task>` through the alias in `.cargo/config.toml`:
//!
//! - `check-docs`: the documentation gates.
//! - `check-dco BASE HEAD`: every commit in `BASE..HEAD` carries a sign-off.
//!
//! Each gate exists because the failure it catches is easy to make and
//! invisible to a reader; the comment on each one names that failure.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use regex::Regex;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("check-docs") => check_docs(),
        Some("check-dco") => match args.as_slice() {
            [_, base, head] => check_dco(base, head),
            _ => usage(),
        },
        _ => usage(),
    };
    process::exit(code);
}

fn usage() -> i32 {
    eprintln!("usage: cargo xtask <check-docs | check-dco BASE HEAD>");
    2
}

/// The repository root: the parent of this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the repository root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

// ── check-docs ─────────────────────────────────────────────────────────────

/// Runs the four documentation gates and reports every failure at once.
fn check_docs() -> i32 {
    let root = repo_root();
    let files = markdown_files(&root);
    let mut failures = Vec::new();
    failures.extend(check_links(&root, &files));
    failures.extend(check_status_lines(&root, &files));
    failures.extend(check_forbidden(&root, &files));
    failures.extend(check_status_tracker(&root));
    for failure in &failures {
        println!("FAIL  {failure}");
    }
    println!(
        "checked {} files: {} failure(s)",
        files.len(),
        failures.len()
    );
    i32::from(!failures.is_empty())
}

/// Every Markdown file the gates look at, plus the two root text files that
/// must obey the naming rule.
fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in ["docs", "rfcs", ".github"] {
        collect_markdown(&root.join(dir), &mut files);
    }
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            }
        }
    }
    for name in ["NOTICE", "CODEOWNERS"] {
        let path = root.join(name);
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    files.dedup();
    files
}

/// Recursively collects Markdown files, skipping build and dependency trees.
fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if matches!(name.as_ref(), ".git" | "target" | "node_modules") {
                continue;
            }
            collect_markdown(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }
}

/// Every relative link resolves to a file that exists. A renamed page
/// silently breaks the map otherwise.
fn check_links(root: &Path, files: &[PathBuf]) -> Vec<String> {
    let link = Regex::new(r"\]\(([^)#\s]+)(?:#[^)]*)?\)").expect("valid regex");
    let mut failures = Vec::new();
    for md in files {
        let text = read(md);
        let dir = md.parent().unwrap_or(root);
        for captures in link.captures_iter(&text) {
            let target = &captures[1];
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
            {
                continue;
            }
            if !dir.join(target).exists() {
                failures.push(format!("broken link in {}: {target}", rel(root, md)));
            }
        }
    }
    failures
}

/// Every page inside a numbered docs directory states its status near the
/// top, so a reader knows whether they hold research, a draft, a decision
/// or a plan.
fn check_status_lines(root: &Path, files: &[PathBuf]) -> Vec<String> {
    let numbered = Regex::new(r"^\d{2}-").expect("valid regex");
    let docs = root.join("docs");
    let mut failures = Vec::new();
    for md in files {
        let Ok(inside) = md.strip_prefix(&docs) else {
            continue;
        };
        let Some(first) = inside.components().next() else {
            continue;
        };
        if !numbered.is_match(&first.as_os_str().to_string_lossy()) {
            continue;
        }
        let has_status = read(md)
            .lines()
            .take(20)
            .any(|line| line.starts_with("Status:"));
        if !has_status {
            failures.push(format!("no Status line near the top of {}", rel(root, md)));
        }
    }
    failures
}

/// Internal product and company names stay out of this public repository;
/// the predecessor engine is referred to as "the baseline engine".
fn check_forbidden(root: &Path, files: &[PathBuf]) -> Vec<String> {
    let patterns = [r"(?i)joishi", r"(?i)softup", r"@jyotisha"];
    let compiled: Vec<Regex> = patterns
        .iter()
        .map(|pattern| Regex::new(pattern).expect("valid regex"))
        .collect();
    let mut failures = Vec::new();
    for md in files {
        let text = read(md);
        for (pattern, regex) in patterns.iter().zip(&compiled) {
            if regex.is_match(&text) {
                failures.push(format!("forbidden term {pattern} in {}", rel(root, md)));
            }
        }
    }
    failures
}

/// The tracker is only useful if it says when it was last true.
fn check_status_tracker(root: &Path) -> Vec<String> {
    let tracker = root.join("docs/STATUS.md");
    if !tracker.is_file() {
        return vec!["docs/STATUS.md is missing".to_string()];
    }
    if !read(&tracker).contains("Last updated:") {
        return vec!["docs/STATUS.md has no 'Last updated:' line".to_string()];
    }
    Vec::new()
}

// ── check-dco ──────────────────────────────────────────────────────────────

/// Every commit in `base..head` carries a `Signed-off-by` line, which is
/// what certifies the Developer Certificate of Origin.
fn check_dco(base: &str, head: &str) -> i32 {
    let signoff = Regex::new(r"(?m)^Signed-off-by: .+ <.+@.+>$").expect("valid regex");
    let range = format!("{base}..{head}");
    let Some(listing) = git(&["rev-list", &range]) else {
        return 2;
    };
    let mut missing = 0;
    for sha in listing.lines().map(str::trim).filter(|sha| !sha.is_empty()) {
        let Some(body) = git(&["show", "-s", "--format=%B", sha]) else {
            return 2;
        };
        if !signoff.is_match(&body) {
            let subject = body.lines().next().unwrap_or("");
            println!("missing sign-off: {sha} {subject}");
            missing += 1;
        }
    }
    println!("checked commits in {range}: {missing} missing sign-off");
    i32::from(missing != 0)
}

/// Runs `git` with the given arguments and returns its standard output, or
/// `None` after printing why it could not.
fn git(args: &[&str]) -> Option<String> {
    match Command::new("git").args(args).output() {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).into_owned())
        }
        Ok(output) => {
            eprintln!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            );
            None
        }
        Err(err) => {
            eprintln!("cannot run git: {err}");
            None
        }
    }
}
