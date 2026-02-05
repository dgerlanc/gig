# Multi-language .gitignore Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add support for combining multiple language templates into a single deduplicated .gitignore file.

**Architecture:** Replace `-l/--lang` flag with positional comma-separated languages. Parse languages, validate all exist, then merge templates by iterating lines and tracking seen patterns in a HashSet to deduplicate.

**Tech Stack:** Rust, pico-args (CLI parsing), include_dir (embedded templates)

---

### Task 1: Parse comma-separated languages

**Files:**
- Modify: `src/main.rs:84-98` (replace `parse_args` function)
- Test: `src/main.rs` (tests module)

**Step 1: Write the failing tests**

Add these tests to the `mod tests` block in `src/main.rs`:

```rust
#[test]
fn test_parse_languages_single() {
    let result = parse_languages("python");
    assert_eq!(result, Ok(vec!["python".to_string()]));
}

#[test]
fn test_parse_languages_multiple() {
    let result = parse_languages("go,godot,emacs");
    assert_eq!(result, Ok(vec!["go".to_string(), "godot".to_string(), "emacs".to_string()]));
}

#[test]
fn test_parse_languages_empty_segment() {
    let result = parse_languages("go,,godot");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty language"));
}

#[test]
fn test_parse_languages_whitespace_trimmed() {
    let result = parse_languages(" go , godot ");
    assert_eq!(result, Ok(vec!["go".to_string(), "godot".to_string()]));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test parse_languages 2>&1`
Expected: FAIL with "cannot find function `parse_languages`"

**Step 3: Write minimal implementation**

Add this function above `parse_args` in `src/main.rs`:

```rust
/// Parse comma-separated language list, validating no empty segments.
fn parse_languages(input: &str) -> Result<Vec<String>, String> {
    let languages: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    if languages.iter().any(|s| s.is_empty()) {
        return Err("empty language in list".to_string());
    }

    Ok(languages)
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test parse_languages 2>&1`
Expected: All 4 tests PASS

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "Add parse_languages function for comma-separated input"
```

---

### Task 2: Add merge_templates function

**Files:**
- Modify: `src/main.rs` (add new function)
- Test: `src/main.rs` (tests module)

**Step 1: Write the failing tests**

Add these tests to the `mod tests` block:

```rust
#[test]
fn test_merge_templates_single() {
    let templates = vec!["# Comment\n*.log\n"];
    let result = merge_templates(&templates);
    assert_eq!(result, "# Comment\n*.log\n");
}

#[test]
fn test_merge_templates_deduplicates_patterns() {
    let templates = vec!["# First\n*.log\n", "# Second\n*.log\n*.txt\n"];
    let result = merge_templates(&templates);
    assert_eq!(result, "# First\n*.log\n# Second\n*.txt\n");
}

#[test]
fn test_merge_templates_preserves_comments() {
    let templates = vec!["# Same comment\n*.a\n", "# Same comment\n*.b\n"];
    let result = merge_templates(&templates);
    assert_eq!(result, "# Same comment\n*.a\n# Same comment\n*.b\n");
}

#[test]
fn test_merge_templates_preserves_blank_lines() {
    let templates = vec!["*.a\n\n*.b\n", "*.c\n\n*.d\n"];
    let result = merge_templates(&templates);
    assert_eq!(result, "*.a\n\n*.b\n*.c\n\n*.d\n");
}

#[test]
fn test_merge_templates_exact_match_only() {
    // *.LOG and *.log are different patterns
    let templates = vec!["*.log\n", "*.LOG\n"];
    let result = merge_templates(&templates);
    assert_eq!(result, "*.log\n*.LOG\n");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test merge_templates 2>&1`
Expected: FAIL with "cannot find function `merge_templates`"

**Step 3: Write minimal implementation**

Add this function after `parse_languages` and add `HashSet` to the imports:

Update imports at top of file:
```rust
use std::collections::{HashMap, HashSet};
```

Add function:
```rust
/// Merge multiple templates, deduplicating patterns but preserving comments and blanks.
fn merge_templates(templates: &[&str]) -> String {
    let mut seen_patterns: HashSet<&str> = HashSet::new();
    let mut output = String::new();

    for template in templates {
        for line in template.lines() {
            let trimmed = line.trim();

            // Comments and blank lines are always included
            if trimmed.is_empty() || trimmed.starts_with('#') {
                output.push_str(line);
                output.push('\n');
                continue;
            }

            // Patterns are deduplicated by exact match
            if seen_patterns.insert(trimmed) {
                output.push_str(line);
                output.push('\n');
            }
        }
    }

    output
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test merge_templates 2>&1`
Expected: All 5 tests PASS

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "Add merge_templates function with pattern deduplication"
```

---

### Task 3: Update CLI to use positional arguments

**Files:**
- Modify: `src/main.rs:11-12` (error message constant)
- Modify: `src/main.rs:13-32` (HELP_MSG)
- Modify: `src/main.rs:84-98` (parse_args function)
- Test: `src/main.rs` (tests module)

**Step 1: Write the failing tests**

Replace the existing `parse_args` tests with these new ones:

```rust
#[test]
fn test_parse_args_single_language() {
    let mut args = pico_args::Arguments::from_vec(vec!["python".into()]);
    let result = parse_args(&mut args);
    assert!(result.is_ok());
    let (langs, output) = result.unwrap();
    assert_eq!(langs, vec!["python".to_string()]);
    assert_eq!(output, PathBuf::from(".gitignore"));
}

#[test]
fn test_parse_args_multiple_languages() {
    let mut args = pico_args::Arguments::from_vec(vec!["go,godot,emacs".into()]);
    let result = parse_args(&mut args);
    assert!(result.is_ok());
    let (langs, output) = result.unwrap();
    assert_eq!(langs, vec!["go".to_string(), "godot".to_string(), "emacs".to_string()]);
    assert_eq!(output, PathBuf::from(".gitignore"));
}

#[test]
fn test_parse_args_with_output_path() {
    let mut args = pico_args::Arguments::from_vec(vec!["rust".into(), "custom.gitignore".into()]);
    let result = parse_args(&mut args);
    assert!(result.is_ok());
    let (langs, output) = result.unwrap();
    assert_eq!(langs, vec!["rust".to_string()]);
    assert_eq!(output, PathBuf::from("custom.gitignore"));
}

#[test]
fn test_parse_args_missing_languages() {
    let mut args = pico_args::Arguments::from_vec(vec![]);
    let result = parse_args(&mut args);
    assert!(result.is_err());
}

#[test]
fn test_parse_args_empty_language_in_list() {
    let mut args = pico_args::Arguments::from_vec(vec!["go,,godot".into()]);
    let result = parse_args(&mut args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty language"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_parse_args 2>&1`
Expected: FAIL (old tests still use `-l` flag format)

**Step 3: Update constants and parse_args**

Update `LANG_REQUIRED_ERR`:
```rust
const LANG_REQUIRED_ERR: &str = "languages required (e.g., gig python or gig go,godot,emacs)";
```

Update `HELP_MSG`:
```rust
const HELP_MSG: &str = r#"gig - generate .gitignore files from GitHub's template collection

Usage:
  gig <languages> [output]

Arguments:
  languages  Comma-separated list of language/tool templates (e.g., python or go,godot,emacs)
  output     Path to write the .gitignore file (default: .gitignore)

Flags:
  --list         List all available language templates
  -h, --help     Show this help message
  -V, --version  Show version information

Examples:
  gig python                   Create .gitignore for Python
  gig go,godot,emacs           Create .gitignore for Go + Godot + Emacs
  gig rust src/.gitignore      Create .gitignore for Rust in src/

Templates are sourced from https://github.com/github/gitignore"#;
```

Update `parse_args` function:
```rust
fn parse_args(args: &mut pico_args::Arguments) -> Result<(Vec<String>, PathBuf), String> {
    // First positional: languages (required)
    let languages_arg: Option<String> = args
        .opt_free_from_str()
        .map_err(|e| e.to_string())?;

    let languages_str = languages_arg.ok_or(LANG_REQUIRED_ERR)?;
    let languages = parse_languages(&languages_str)?;

    // Second positional: output path (optional)
    let output: PathBuf = args
        .opt_free_from_str()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT));

    Ok((languages, output))
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_parse_args 2>&1`
Expected: All 5 tests PASS

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "Update CLI to use positional comma-separated languages"
```

---

### Task 4: Update main() to handle multiple languages

**Files:**
- Modify: `src/main.rs:37-82` (main function)

**Step 1: No new unit tests needed**

This task integrates existing tested functions. We'll verify with manual testing.

**Step 2: Update main function**

Replace the main function:
```rust
fn main() {
    let mut args = pico_args::Arguments::from_env();

    // Handle --help / -h
    if args.contains(["-h", "--help"]) || std::env::args().len() == 1 {
        print_usage();
        process::exit(0);
    }

    // Handle --version / -V
    if args.contains(["-V", "--version"]) {
        println!("gig {}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    // Handle --list
    if args.contains("--list") {
        list_languages();
        process::exit(0);
    }

    // Parse languages and output path
    let (languages, output) = match parse_args(&mut args) {
        Ok((l, o)) => (l, o),
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    // Get template content for each language
    let mut templates: Vec<&'static str> = Vec::new();
    for lang in &languages {
        match get_template(lang) {
            Ok(content) => templates.push(content),
            Err(e) => {
                eprintln!("error: {e}");
                eprintln!("\nRun 'gig --list' to see available languages.");
                process::exit(1);
            }
        }
    }

    // Merge templates and write output
    let content = merge_templates(&templates);
    if let Err(e) = write_output(&output, &content) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
```

**Step 3: Run all tests**

Run: `cargo test 2>&1`
Expected: All tests PASS

**Step 4: Manual verification**

Run: `cargo run -- python 2>&1 | head -5`
Expected: Error about .gitignore already exists OR first 5 lines of Python template

Run: `cargo run -- --help 2>&1`
Expected: Updated help message with new syntax

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "Integrate multi-language support in main()"
```

---

### Task 5: Remove old tests and clean up

**Files:**
- Modify: `src/main.rs` (tests module - remove obsolete tests)

**Step 1: Remove obsolete tests**

Delete these old tests from the tests module:
- `test_parse_args_with_lang`
- `test_parse_args_with_lang_and_output`
- `test_parse_args_long_flag`
- `test_parse_args_missing_lang`
- `test_parse_args_lang_flag_without_value`

These tested the old `-l` flag behavior which no longer exists.

**Step 2: Run all tests**

Run: `cargo test 2>&1`
Expected: All remaining tests PASS

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "Remove obsolete -l flag tests"
```

---

### Task 6: Add integration test for multi-language

**Files:**
- Modify: `src/main.rs` (tests module)

**Step 1: Write the integration test**

Add this test to verify end-to-end multi-language behavior:

```rust
#[test]
fn test_multi_language_deduplication() {
    // Get two templates that likely share some patterns
    let go = get_template("go").unwrap();
    let rust = get_template("rust").unwrap();

    let merged = merge_templates(&[go, rust]);

    // Verify merged content contains patterns from both
    assert!(merged.contains("*.exe"), "should contain Go's *.exe pattern");

    // Count occurrences of *.exe - should only appear once
    let exe_count = merged.lines().filter(|l| l.trim() == "*.exe").count();
    assert_eq!(exe_count, 1, "*.exe should only appear once after deduplication");
}
```

**Step 2: Run the test**

Run: `cargo test test_multi_language_deduplication 2>&1`
Expected: PASS

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "Add integration test for multi-language deduplication"
```

---

### Task 7: Update CLAUDE.md with new CLI usage

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update the CLI Usage section**

Update the CLI Usage section in CLAUDE.md:

```markdown
## CLI Usage

```bash
gig <languages> [output]      # Generate .gitignore (output defaults to .gitignore)
gig python                    # Single language
gig go,godot,emacs            # Multiple languages, comma-separated
gig --list                    # List available templates
gig --help                    # Show help
gig --version                 # Show version
```
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "Update CLAUDE.md with new multi-language CLI syntax"
```

---

## Summary

| Task | Description | Tests |
|------|-------------|-------|
| 1 | Parse comma-separated languages | 4 unit tests |
| 2 | Merge templates with deduplication | 5 unit tests |
| 3 | Update CLI to positional args | 5 unit tests |
| 4 | Integrate in main() | Manual verification |
| 5 | Remove obsolete tests | Cleanup |
| 6 | Integration test | 1 integration test |
| 7 | Update documentation | N/A |

**Total new tests:** 15 (replacing 5 obsolete tests)
