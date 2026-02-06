# Nested Template Indexing

Index all templates in the `templates/` directory, not just top-level ones.

## Background

Currently `build_index()` uses `TEMPLATES.files()` which only iterates top-level
files (157 templates). There are 142 more templates in subdirectories
(`community/`, `Global/`) that are ignored. Total: 299 templates.

Three name collisions exist across directories:
- `AL`: top-level and `Global/AL.gitignore`
- `Racket`: top-level and `community/Racket.gitignore`
- `ColdBox`: `community/CFML/ColdBox.gitignore` and `community/BoxLang/ColdBox.gitignore`

## Design Decisions

- **Build-time flattening**: A `build.rs` script copies all templates into a flat
  `$OUT_DIR/templates/` directory before compilation
- **Collision handling**: Top-level keeps its name; nested collisions get prefixed
  with their parent directory (e.g., `Global-AL.gitignore`, `CFML-ColdBox.gitignore`)
- **No prefix matching**: Removed entirely to simplify lookup
- **Minimal Rust changes**: `build_index()` stays unchanged (flat directory scan)

## Approach

### `build.rs` (new)

Walks `templates/` recursively at build time:
1. Collects all `.gitignore` files with their paths and bare names
2. Groups by bare name to detect collisions
3. Unique names: copied as-is
4. Collisions: top-level keeps its name, nested get prefixed with parent dir name
5. Fails the build on secondary collisions (after prefixing)
6. Copies all files to `$OUT_DIR/templates/`

### `src/main.rs` changes

- `include_dir!` path changes from `$CARGO_MANIFEST_DIR/templates` to `$OUT_DIR/templates`
- `get_template()` simplified to exact-match only (prefix matching removed)
- `build_index()` unchanged
- `list_languages()` refactored for testability

### Result for current collisions

- `AL` -> `al` (top-level wins), `global-al` (prefixed)
- `Racket` -> `racket` (top-level wins), `community-racket` (prefixed)
- `ColdBox` -> `cfml-coldbox` and `boxlang-coldbox` (both prefixed, no bare name)

## Testing

Remove existing prefix-matching tests (`test_get_template_prefix_match`,
`test_get_template_ambiguous`).

### New tests

- `test_build_index_includes_nested_templates` -- index size > 200
- `test_collision_top_level_wins` -- `al` returns top-level, `global-al` returns Global
- `test_collision_same_priority_both_prefixed` -- `cfml-coldbox` and `boxlang-coldbox` work
- `test_unique_nested_template_bare_name` -- `vue` accessible by bare name
- `test_nested_template_content_not_empty` -- spot-check content
- `test_list_no_duplicates` -- no duplicate entries in --list
- `test_list_includes_nested_templates` -- nested templates appear in --list
