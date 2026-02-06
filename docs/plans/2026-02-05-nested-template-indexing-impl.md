# Nested Template Indexing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Index all 299 `.gitignore` templates (not just top-level 157) by flattening the directory at build time.

**Architecture:** A `build.rs` script walks `templates/` recursively and copies all `.gitignore` files into a flat staging directory in `$OUT_DIR/templates/`. Collisions are resolved by prefixing with the parent directory name. `include_dir!` points at the flat staging directory. The existing `build_index()` stays unchanged (flat scan). Prefix matching is removed.

**Tech Stack:** Rust, `include_dir` 0.7, `build.rs`

**Working directory:** `/Users/dgerlanc/code/gig/.worktrees/nested-template-indexing`

---

### Task 1: Remove prefix matching from `get_template()` and delete old tests

This task simplifies `get_template()` to exact-match-only and removes the two prefix-matching tests. Standalone simplification that clears the way for later changes.

**Files:**
- Modify: `src/main.rs:157-183` (`get_template` function)
- Modify: `src/main.rs:262-266` (delete `test_get_template_prefix_match`)
- Modify: `src/main.rs:276-303` (delete `test_get_template_ambiguous`)

**Step 1: Delete the two prefix-matching tests**

Remove `test_get_template_prefix_match` (lines 262-266) and `test_get_template_ambiguous` (lines 276-303) entirely from the `tests` module.

**Step 2: Simplify `get_template()` to exact-match only**

Replace the current `get_template` function (lines 157-183) with:

```rust
/// Get template content for a language (case-insensitive exact match).
fn get_template(lang: &str) -> Result<&'static str, String> {
    let index = &*INDEX;
    let key = lang.to_lowercase();

    index
        .get(&key)
        .copied()
        .ok_or_else(|| format!("no template found for language \"{lang}\""))
}
```

**Step 3: Run tests to verify all pass**

Run: `cargo test`
Expected: All remaining 24 tests pass.

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "Remove prefix matching from get_template

Simplifies lookup to exact match only, preparing for
nested template indexing."
```

---

### Task 2: Add `build.rs` that flattens templates into `$OUT_DIR`

Create a build script that walks `templates/` recursively, copies all `.gitignore` files into a flat `$OUT_DIR/templates/` directory, and resolves name collisions by prefixing with the parent directory.

**Files:**
- Create: `build.rs`

**Step 1: Write failing test for a nested template**

Add to the `tests` module in `src/main.rs`:

```rust
#[test]
fn test_build_index_includes_nested_templates() {
    let index = build_index();
    // With flattened nested templates, we should have many more than top-level only
    assert!(
        index.len() > 200,
        "index should include nested templates, got {} entries",
        index.len()
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_build_index_includes_nested`
Expected: FAIL (index only has ~157 entries from top-level)

**Step 3: Create `build.rs`**

Create `build.rs` in the project root (same level as `Cargo.toml`):

```rust
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const GITIGNORE_SUFFIX: &str = ".gitignore";

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let src_dir = Path::new("templates");
    let dest_dir = out_dir.join("templates");

    // Clean and recreate destination
    if dest_dir.exists() {
        fs::remove_dir_all(&dest_dir).unwrap();
    }
    fs::create_dir_all(&dest_dir).unwrap();

    // Collect all .gitignore files recursively
    let mut templates: Vec<(PathBuf, String)> = Vec::new(); // (source_path, bare_name)
    collect_templates(src_dir, &mut templates);

    // Group by bare name (lowercased) to detect collisions
    let mut by_name: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for (path, bare_name) in &templates {
        by_name
            .entry(bare_name.to_lowercase())
            .or_default()
            .push(path.clone());
    }

    // Determine destination filename for each template
    let mut dest_names: HashMap<PathBuf, String> = HashMap::new();
    let mut final_names: HashMap<String, PathBuf> = HashMap::new();

    for (path, bare_name) in &templates {
        let key = bare_name.to_lowercase();
        let group = &by_name[&key];

        let dest_name = if group.len() == 1 {
            // Unique name — use as-is
            format!("{bare_name}{GITIGNORE_SUFFIX}")
        } else {
            // Collision — check if this is a top-level file
            let is_top_level = path.parent() == Some(Path::new("templates"));
            if is_top_level {
                // Top-level keeps its original name
                format!("{bare_name}{GITIGNORE_SUFFIX}")
            } else {
                // Nested: prefix with parent directory name
                let parent = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .expect("nested template should have a parent directory");
                format!("{parent}-{bare_name}{GITIGNORE_SUFFIX}")
            }
        };

        // Check for secondary collisions after prefixing
        let dest_lower = dest_name.to_lowercase();
        if let Some(existing) = final_names.get(&dest_lower) {
            panic!(
                "Secondary collision: {} and {} both map to {}",
                existing.display(),
                path.display(),
                dest_name
            );
        }
        final_names.insert(dest_lower, path.clone());
        dest_names.insert(path.clone(), dest_name);
    }

    // Copy files to destination
    for (src_path, dest_name) in &dest_names {
        let dest_path = dest_dir.join(dest_name);
        fs::copy(src_path, dest_path).unwrap();
    }

    // Tell Cargo to re-run if templates change
    println!("cargo::rerun-if-changed=templates");
}

fn collect_templates(dir: &Path, templates: &mut Vec<(PathBuf, String)>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            collect_templates(&path, templates);
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(bare) = name.strip_suffix(GITIGNORE_SUFFIX) {
                if !bare.is_empty() {
                    templates.push((path, bare.to_string()));
                }
            }
        }
    }
}
```

**Step 4: Update `include_dir!` path in `src/main.rs`**

Change line 34 from:

```rust
static TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");
```

to:

```rust
static TEMPLATES: Dir<'_> = include_dir!("$OUT_DIR/templates");
```

**Step 5: Run the failing test**

Run: `cargo test test_build_index_includes_nested`
Expected: PASS (the index now includes all ~296 flattened templates)

**Step 6: Run all tests**

Run: `cargo test`
Expected: All 25 tests pass.

**Step 7: Commit**

```bash
git add build.rs src/main.rs
git commit -m "Add build.rs to flatten nested templates

Walk templates/ recursively at build time, copy to flat
OUT_DIR/templates/. Prefix with parent dir on collision."
```

---

### Task 3: Add tests for collision handling

Verify that the 3 known collisions are handled correctly: top-level wins for AL and Racket, ColdBox gets parent-dir prefixed.

**Files:**
- Modify: `src/main.rs` (tests module)

**Step 1: Add the tests**

```rust
#[test]
fn test_collision_top_level_wins() {
    // AL exists at top-level and Global. Top-level should keep bare name.
    let result = get_template("al");
    assert!(result.is_ok(), "top-level 'al' should be accessible by bare name");

    let result_prefixed = get_template("global-al");
    assert!(
        result_prefixed.is_ok(),
        "Global/AL should be accessible as 'global-al'"
    );

    // They should be different templates
    assert_ne!(
        result.unwrap(),
        result_prefixed.unwrap(),
        "top-level AL and Global AL should have different content"
    );
}

#[test]
fn test_collision_same_priority_both_prefixed() {
    // ColdBox exists in community/CFML and community/BoxLang (same priority)
    // Neither should have bare name; both should be prefixed
    let cfml = get_template("cfml-coldbox");
    assert!(cfml.is_ok(), "CFML/ColdBox should be accessible as 'cfml-coldbox'");

    let boxlang = get_template("boxlang-coldbox");
    assert!(
        boxlang.is_ok(),
        "BoxLang/ColdBox should be accessible as 'boxlang-coldbox'"
    );
}

#[test]
fn test_unique_nested_template_bare_name() {
    // Vue is unique (only in community/JavaScript/), should be accessible by bare name
    let result = get_template("vue");
    assert!(result.is_ok(), "unique nested template 'vue' should have bare name");
}

#[test]
fn test_nested_template_content_not_empty() {
    // Spot-check that nested templates have actual content
    let vue = get_template("vue").unwrap();
    assert!(!vue.is_empty(), "vue template should have content");

    let macos = get_template("macos").unwrap();
    assert!(!macos.is_empty(), "macos template should have content");
}
```

**Step 2: Run the tests**

Run: `cargo test test_collision`
Expected: All collision tests PASS.

Run: `cargo test test_unique_nested`
Expected: PASS.

Run: `cargo test test_nested_template_content`
Expected: PASS.

**Step 3: Run all tests**

Run: `cargo test`
Expected: All tests pass.

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "Add tests for collision handling

Verify top-level priority, parent-dir prefixing, and
bare name access for unique nested templates."
```

---

### Task 4: Verify `--list` output and add test

The `--list` output should now include all flattened templates. Since `build_index()` is unchanged and the flat directory has all templates, `list_languages()` should just work. Add a test to verify no duplicates and expected entries.

**Files:**
- Modify: `src/main.rs` (refactor `list_languages` + add tests)

**Step 1: Extract testable function from `list_languages()`**

Replace the existing `list_languages()` (lines 186-194) with:

```rust
/// Get the sorted list of available template keys.
fn get_language_list() -> Vec<String> {
    let index = &*INDEX;
    let mut langs: Vec<_> = index.keys().cloned().collect();
    langs.sort_unstable();
    langs
}

/// List all available languages.
fn list_languages() {
    for lang in get_language_list() {
        println!("{lang}");
    }
}
```

**Step 2: Add tests**

```rust
#[test]
fn test_list_no_duplicates() {
    let list = get_language_list();
    let unique: HashSet<&String> = list.iter().collect();
    assert_eq!(list.len(), unique.len(), "list should have no duplicates");
}

#[test]
fn test_list_includes_nested_templates() {
    let list = get_language_list();
    assert!(
        list.contains(&"vue".to_string()),
        "list should include nested template 'vue'"
    );
    assert!(
        list.contains(&"macos".to_string()),
        "list should include Global template 'macos'"
    );
    assert!(
        list.contains(&"cfml-coldbox".to_string()),
        "list should include prefixed collision 'cfml-coldbox'"
    );
}
```

**Step 3: Run tests**

Run: `cargo test test_list`
Expected: All list tests PASS.

**Step 4: Run all tests**

Run: `cargo test`
Expected: All tests pass.

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "Add tests for --list with nested templates

Extract get_language_list() for testability. Verify no
duplicates and nested templates appear in output."
```
