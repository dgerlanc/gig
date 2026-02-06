use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const GITIGNORE_SUFFIX: &str = ".gitignore";

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let src_dir = Path::new("templates");
    let dest_dir = out_dir.join("templates");

    // Clone templates repo if the directory doesn't exist
    if !src_dir.exists() {
        let status = Command::new("git")
            .args([
                "clone",
                "--depth=1",
                "https://github.com/github/gitignore.git",
                "templates",
            ])
            .status()
            .expect("Failed to execute git clone");

        if !status.success() {
            panic!("Failed to clone gitignore templates repository");
        }

        // Remove the .git directory to avoid nested repo issues
        let git_dir = src_dir.join(".git");
        if git_dir.exists() {
            fs::remove_dir_all(&git_dir).expect("Failed to remove .git directory");
        }
    }

    // Clean and recreate destination
    let _ = fs::remove_dir_all(&dest_dir);
    fs::create_dir_all(&dest_dir).unwrap();

    // Collect all .gitignore files recursively
    let mut templates: Vec<(PathBuf, String)> = Vec::new(); // (source_path, bare_name)
    collect_templates(src_dir, &mut templates);

    // Build destination filename using scope-based prefixing
    for (src_path, bare_name) in &templates {
        let rel = src_path
            .strip_prefix(src_dir)
            .expect("template should be under src_dir");

        let dest_name = compute_dest_name(rel, bare_name);
        let dest_path = dest_dir.join(&dest_name);
        fs::copy(src_path, dest_path).unwrap();
    }

    // Tell Cargo to re-run if templates change
    println!("cargo::rerun-if-changed=templates");
}

/// Compute the destination filename based on the template's scope.
///
/// - Top-level: `{name}.gitignore`
/// - Global/: `global.{name}.gitignore`
/// - community/{subcategory}/: `community.{subcategory}.{name}.gitignore`
/// - community/ (direct): `community.{name}.gitignore`
fn compute_dest_name(rel_path: &Path, bare_name: &str) -> String {
    let components: Vec<&str> = rel_path
        .parent()
        .unwrap_or(Path::new(""))
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    if components.is_empty() {
        // Top-level template
        format!("{bare_name}{GITIGNORE_SUFFIX}")
    } else if components[0].eq_ignore_ascii_case("global") {
        format!("global.{bare_name}{GITIGNORE_SUFFIX}")
    } else if components[0].eq_ignore_ascii_case("community") {
        if components.len() > 1 {
            // community/{subcategory}/{name}.gitignore
            let subcategory = components[1];
            format!("community.{subcategory}.{bare_name}{GITIGNORE_SUFFIX}")
        } else {
            // community/{name}.gitignore (no subcategory)
            format!("community.{bare_name}{GITIGNORE_SUFFIX}")
        }
    } else {
        // Unknown subdirectory — treat like top-level with prefix
        let prefix = components.join(".");
        format!("{prefix}.{bare_name}{GITIGNORE_SUFFIX}")
    }
}

fn collect_templates(dir: &Path, templates: &mut Vec<(PathBuf, String)>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read directory {}: {e}", dir.display()));

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            collect_templates(&path, templates);
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && let Some(bare) = name.strip_suffix(GITIGNORE_SUFFIX)
            && !bare.is_empty()
        {
            let bare_owned = bare.to_string();
            templates.push((path, bare_owned));
        }
    }
}
