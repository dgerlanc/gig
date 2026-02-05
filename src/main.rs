use include_dir::{Dir, include_dir};
use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::LazyLock;

const DEFAULT_OUTPUT: &str = ".gitignore";
const GITIGNORE_SUFFIX: &str = ".gitignore";
const LANG_REQUIRED_ERR: &str = "languages required (e.g., gig python or gig go,godot,node)";

const HELP_MSG: &str = r#"gig - generate .gitignore files from GitHub's template collection

Usage:
  gig <languages> [output]

Arguments:
  languages  Comma-separated list of language/tool templates (e.g., python or go,godot,node)
  output     Path to write the .gitignore file (default: .gitignore)

Flags:
  --list         List all available language templates
  -h, --help     Show this help message
  -V, --version  Show version information

Examples:
  gig python                   Create .gitignore for Python
  gig go,godot,node            Create .gitignore for Go + Godot + Node
  gig rust src/.gitignore      Create .gitignore for Rust in src/

Templates are sourced from https://github.com/github/gitignore"#;

static TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");
static INDEX: LazyLock<HashMap<String, &'static str>> = LazyLock::new(build_index);

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
    let mut templates: Vec<&str> = Vec::new();
    for lang in &languages {
        match get_template(lang) {
            Ok(c) => templates.push(c),
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

/// Build an index mapping lowercase language names to their template content.
fn build_index() -> HashMap<String, &'static str> {
    TEMPLATES
        .files()
        .filter_map(|file| {
            let name = file.path().file_name()?.to_str()?;
            let lang = name
                .strip_suffix(GITIGNORE_SUFFIX)
                .filter(|s| !s.is_empty())?;
            let content = file.contents_utf8()?;
            Some((lang.to_lowercase(), content))
        })
        .collect()
}

/// Get template content for a language with case-insensitive and prefix matching.
fn get_template(lang: &str) -> Result<&'static str, String> {
    let index = &*INDEX;
    let key = lang.to_lowercase();

    // Exact match
    if let Some(content) = index.get(&key) {
        return Ok(content);
    }

    // Prefix match
    let matches: Vec<&String> = index.keys().filter(|k| k.starts_with(&key)).collect();

    match matches.as_slice() {
        [] => Err(format!("no template found for language \"{lang}\"")),
        [single] => Ok(index[*single]),
        multiple => {
            let mut sorted: Vec<_> = multiple.iter().map(|s| s.as_str()).collect();
            sorted.sort_unstable();
            Err(format!(
                "ambiguous language \"{}\"; matches: {}",
                lang,
                sorted.join(", ")
            ))
        }
    }
}

/// List all available languages.
fn list_languages() {
    let index = &*INDEX;
    let mut langs: Vec<_> = index.keys().collect();
    langs.sort_unstable();

    for lang in langs {
        println!("{lang}");
    }
}

/// Write content to a file, refusing to overwrite existing files.
fn write_output(path: &Path, content: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| {
            if e.kind() == ErrorKind::AlreadyExists {
                format!(
                    "file {} already exists; remove it first or choose a different path",
                    path.display()
                )
            } else {
                e.to_string()
            }
        })?;
    file.write_all(content.as_bytes())
        .map_err(|e| e.to_string())
}

fn print_usage() {
    println!("{HELP_MSG}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gig_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_build_index_has_templates() {
        let index = build_index();
        assert!(!index.is_empty(), "index should contain embedded templates");
    }

    #[test]
    fn test_build_index_lowercase_keys() {
        let index = build_index();
        for key in index.keys() {
            assert_eq!(key, &key.to_lowercase(), "all keys should be lowercase");
        }
    }

    #[test]
    fn test_get_template_exact_match() {
        let result = get_template("python");
        assert!(result.is_ok(), "should find python template");
    }

    #[test]
    fn test_get_template_case_insensitive() {
        let lower = get_template("python").unwrap();
        let upper = get_template("Python").unwrap();
        let mixed = get_template("PYTHON").unwrap();

        assert_eq!(lower, upper);
        assert_eq!(lower, mixed);
    }

    #[test]
    fn test_get_template_prefix_match() {
        // "pyth" should uniquely match "python"
        let result = get_template("pyth");
        assert!(result.is_ok(), "prefix 'pyth' should match python");
    }

    #[test]
    fn test_get_template_not_found() {
        let result = get_template("nonexistentlanguage12345");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no template found"));
    }

    #[test]
    fn test_get_template_ambiguous() {
        let index = build_index();
        // Find a prefix that matches multiple templates but isn't an exact match itself
        let mut prefix_matches: HashMap<String, Vec<String>> = HashMap::new();
        for key in index.keys() {
            if key.len() >= 2 {
                let prefix = &key[..2];
                prefix_matches
                    .entry(prefix.to_string())
                    .or_default()
                    .push(key.clone());
            }
        }

        // Find an ambiguous prefix (one that matches multiple and isn't an exact key)
        for (prefix, matches) in prefix_matches {
            if matches.len() > 1 && !index.contains_key(&prefix) {
                let result = get_template(&prefix);
                assert!(
                    result.is_err(),
                    "should be ambiguous for prefix '{}'",
                    prefix
                );
                assert!(
                    result.unwrap_err().contains("ambiguous"),
                    "error should mention ambiguous"
                );
                return;
            }
        }
        panic!("No ambiguous prefix found in templates - test needs updating");
    }

    #[test]
    fn test_write_output_creates_file() {
        let dir = test_dir();
        let path = dir.join("test.gitignore");

        let result = write_output(&path, "# test content\n");
        assert!(result.is_ok());
        assert!(path.exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), "# test content\n");

        // Cleanup
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_write_output_refuses_overwrite() {
        let dir = test_dir();
        let path = dir.join("existing.gitignore");

        // Create existing file
        fs::write(&path, "existing content").unwrap();

        let result = write_output(&path, "new content");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));

        // Verify content unchanged
        assert_eq!(fs::read_to_string(&path).unwrap(), "existing content");

        // Cleanup
        fs::remove_dir_all(&dir).ok();
    }

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

    #[test]
    fn test_version_string() {
        // Verify the version macro returns a valid semver string
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty(), "version should not be empty");
        // Basic semver format check (x.y.z)
        let parts: Vec<&str> = version.split('.').collect();
        assert!(parts.len() >= 2, "version should have at least major.minor");
    }

    #[test]
    fn test_help_message_includes_version_flag() {
        // Verify the help message documents the version flag
        assert!(
            HELP_MSG.contains("-V, --version"),
            "help message should document -V/--version flag"
        );
    }

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
}
