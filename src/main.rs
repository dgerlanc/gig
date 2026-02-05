use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::process;
use std::sync::LazyLock;

const DEFAULT_OUTPUT: &str = ".gitignore";
const GITIGNORE_SUFFIX: &str = ".gitignore";

static TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");
static INDEX: LazyLock<HashMap<String, &'static str>> = LazyLock::new(build_index);

fn main() {
    let mut args = pico_args::Arguments::from_env();

    // Handle --help / -h
    if args.contains(["-h", "--help"]) || std::env::args().len() == 1 {
        print_usage();
        process::exit(0);
    }

    // Handle --list
    if args.contains("--list") {
        if let Err(e) = list_languages() {
            eprintln!("error: {e}");
            process::exit(1);
        }
        process::exit(0);
    }

    // Parse -l/--lang and output path
    let (lang, output) = match parse_args(&mut args) {
        Ok((l, o)) => (l, o),
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    // Get template content
    let content = match get_template(&lang) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("\nRun 'gig --list' to see available languages.");
            process::exit(1);
        }
    };

    // Write output
    if let Err(e) = write_output(&output, content) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn parse_args(args: &mut pico_args::Arguments) -> Result<(String, String), String> {
    let lang: Option<String> = args
        .opt_value_from_str(["-l", "--lang"])
        .map_err(|e| e.to_string())?;

    let lang = lang.ok_or("language is required; use -l <language>")?;

    // Get positional argument (output path), default to .gitignore
    let output = args
        .opt_free_from_str()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| DEFAULT_OUTPUT.to_string());

    Ok((lang, output))
}

/// Build an index mapping lowercase language names to their template content.
fn build_index() -> HashMap<String, &'static str> {
    TEMPLATES
        .files()
        .filter_map(|file| {
            let name = file.path().file_name()?.to_str()?;
            let lang = name.strip_suffix(GITIGNORE_SUFFIX).filter(|s| !s.is_empty())?;
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
fn list_languages() -> Result<(), String> {
    let index = &*INDEX;

    if index.is_empty() {
        return Err(
            "no templates embedded. Please contact the project maintainers".to_string(),
        );
    }

    let mut langs: Vec<_> = index.keys().collect();
    langs.sort_unstable();

    println!("Available languages ({}):\n", langs.len());
    for lang in langs {
        println!("  {lang}");
    }

    Ok(())
}

/// Write content to a file, refusing to overwrite existing files.
fn write_output(path: &str, content: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| {
            if e.kind() == ErrorKind::AlreadyExists {
                format!("file {path} already exists; remove it first or choose a different path")
            } else {
                e.to_string()
            }
        })?;
    file.write_all(content.as_bytes()).map_err(|e| e.to_string())
}

fn print_usage() {
    println!(
        r#"gig - generate .gitignore files from GitHub's template collection

Usage:
  gig -l <language> [output]

Arguments:
  output    Path to write the .gitignore file (default: .gitignore)

Flags:
  -l, --lang     Language template to use (required)
  --list         List all available language templates
  -h, --help     Show this help message

Examples:
  gig -l python                  Create .gitignore for Python in current directory
  gig -l go .gitignore           Same as above, explicit output path
  gig -l rust src/.gitignore     Create .gitignore for Rust in src/

Templates are sourced from https://github.com/github/gitignore"#
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn test_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gig_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_build_index_has_templates() {
        let index = build_index();
        // Should have templates if valid template files are embedded
        // (files ending in .gitignore that aren't just ".gitignore")
        let has_valid_templates = TEMPLATES.files().any(|f| {
            f.path()
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|name| name.strip_suffix(GITIGNORE_SUFFIX))
                .is_some_and(|lang| !lang.is_empty())
        });
        if has_valid_templates {
            assert!(!index.is_empty(), "index should not be empty when templates exist");
        }
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
        let index = build_index();
        if index.contains_key("python") {
            let result = get_template("python");
            assert!(result.is_ok(), "should find python template");
        }
    }

    #[test]
    fn test_get_template_case_insensitive() {
        let index = build_index();
        if index.contains_key("python") {
            let lower = get_template("python").unwrap();
            let upper = get_template("Python").unwrap();
            let mixed = get_template("PYTHON").unwrap();

            assert_eq!(lower, upper);
            assert_eq!(lower, mixed);
        }
    }

    #[test]
    fn test_get_template_prefix_match() {
        let index = build_index();
        // "py" should match "python" if it's the only match starting with "py"
        if index.contains_key("python") && !index.keys().any(|k| k != "python" && k.starts_with("py")) {
            let result = get_template("py");
            assert!(result.is_ok(), "prefix 'py' should match python");
        }
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
                assert!(result.is_err(), "should be ambiguous for prefix '{}'", prefix);
                assert!(
                    result.unwrap_err().contains("ambiguous"),
                    "error should mention ambiguous"
                );
                return;
            }
        }
        // If we get here, no ambiguous prefix was found - skip the test
    }

    #[test]
    fn test_write_output_creates_file() {
        let dir = test_dir();
        let path = dir.join("test.gitignore");
        let path_str = path.to_str().unwrap();

        let result = write_output(path_str, "# test content\n");
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
        let path_str = path.to_str().unwrap();

        // Create existing file
        fs::write(&path, "existing content").unwrap();

        let result = write_output(path_str, "new content");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));

        // Verify content unchanged
        assert_eq!(fs::read_to_string(&path).unwrap(), "existing content");

        // Cleanup
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_parse_args_with_lang() {
        let mut args = pico_args::Arguments::from_vec(vec![
            "-l".into(),
            "python".into(),
        ]);
        let result = parse_args(&mut args);
        assert!(result.is_ok());
        let (lang, output) = result.unwrap();
        assert_eq!(lang, "python");
        assert_eq!(output, ".gitignore");
    }

    #[test]
    fn test_parse_args_with_lang_and_output() {
        let mut args = pico_args::Arguments::from_vec(vec![
            "-l".into(),
            "rust".into(),
            "custom.gitignore".into(),
        ]);
        let result = parse_args(&mut args);
        assert!(result.is_ok());
        let (lang, output) = result.unwrap();
        assert_eq!(lang, "rust");
        assert_eq!(output, "custom.gitignore");
    }

    #[test]
    fn test_parse_args_long_flag() {
        let mut args = pico_args::Arguments::from_vec(vec![
            "--lang".into(),
            "go".into(),
        ]);
        let result = parse_args(&mut args);
        assert!(result.is_ok());
        let (lang, _) = result.unwrap();
        assert_eq!(lang, "go");
    }

    #[test]
    fn test_parse_args_missing_lang() {
        let mut args = pico_args::Arguments::from_vec(vec![]);
        let result = parse_args(&mut args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("language is required"));
    }
}
