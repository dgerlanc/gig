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

- **Hybrid namespace**: Flat by default (bare name), path-based for disambiguation
- **Priority order for collisions**: top-level (0) > community (1) > Global (2)
- **`--list` output**: Flat sorted list, path-qualified only for collisions
- **No prefix matching**: Removed entirely to simplify lookup

## Index Structure

Same type as today: `HashMap<String, &'static str>`.

For every `.gitignore` file found recursively, compute:
- **path_key**: Relative path from `templates/`, lowercased, `.gitignore` stripped.
  E.g., `community/javascript/vue`, `global/macos`, `python`.
- **bare_name**: Filename only, lowercased, `.gitignore` stripped.
  E.g., `vue`, `macos`, `python`.

Insertion rules:
1. Every template gets its **path_key** inserted (always).
2. If a bare name is unique across all templates, it gets inserted as a shortcut.
3. If there's a collision, the highest-priority template wins the bare name key,
   but only if exactly one template exists at that priority level.
4. If multiple templates collide at the same priority (like `ColdBox`), none gets
   a bare name key.

Result for current collisions:
- `AL` -> keys: `al` (top-level wins), `global/al`
- `Racket` -> keys: `racket` (top-level wins), `community/racket`
- `ColdBox` -> keys: `community/cfml/coldbox`, `community/boxlang/coldbox` (no bare key)

## Lookup Behavior

`get_template()` becomes:

1. **Exact match** on the index key -- handles `gig python`, `gig global/al`, etc.
2. **Miss** -- check a collision map (`bare_name -> Vec<path_key>`) to distinguish:
   - Collision: `ambiguous template "coldbox"; specify one of: community/boxlang/coldbox, community/cfml/coldbox`
   - Genuinely missing: `no template found for language "foo"`

## `--list` Output

Print the **shortest usable key** for each template:

1. Iterate over all path keys (one per actual template file).
2. If a bare name shortcut exists in the index for that template, print the bare name.
3. Otherwise, print the path key.

Produces a clean deduplicated list:
```
actionscript
ada
al
altiumdesigner
...
community/boxlang/coldbox
community/cfml/coldbox
...
global/al
...
vue
```

Requires a `HashSet<String>` of all path keys to distinguish path keys from bare
name shortcuts during iteration.

## Testing

Remove existing prefix-matching tests (`test_get_template_prefix_match`,
`test_get_template_ambiguous`).

### New tests

**Index building:**
- `test_build_index_includes_nested_templates` -- index size > top-level count
- `test_build_index_has_path_keys` -- `global/macos`, `community/javascript/vue` exist
- `test_build_index_bare_name_for_unique_nested` -- `vue` exists as a key
- `test_build_index_no_bare_name_for_same_priority_collision` -- `coldbox` does NOT exist

**Lookup:**
- `test_get_template_path_key_lookup` -- `get_template("global/al")` succeeds
- `test_get_template_bare_name_shortcut` -- `get_template("vue")` succeeds
- `test_get_template_collision_priority` -- `get_template("al")` returns top-level content
- `test_get_template_collision_same_priority_error` -- `get_template("coldbox")` errors with both paths
- `test_get_template_not_found_error` -- `get_template("nonexistent")` returns "no template found"

**List:**
- `test_list_no_duplicates` -- no duplicate entries
- `test_list_shows_shortest_key` -- `vue` appears, `community/boxlang/coldbox` appears
