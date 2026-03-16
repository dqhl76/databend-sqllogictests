## Goal

Add a new glob-based test selection mode using `--run` and `--skip`, while preserving the existing `--suites`, `--run_dir`, `--run_file`, `--skip_dir`, and `--skip_file` behavior for compatibility.

## Design

- Keep the legacy selectors unchanged in behavior.
- Introduce a new selector mode:
  - `--run`: include test files by filesystem glob patterns
  - `--skip`: exclude test files by filesystem glob patterns
- New mode does not depend on `--suites`.
- New mode and legacy selector mode should not be mixed in one invocation.
- Glob patterns should support both file and directory matches:
  - matching a file includes that file
  - matching a directory includes or excludes all files under it recursively

## Implementation Steps

1. Update argument definitions so `--run` / `--skip` are the new recommended selectors and mark them as conflicting with the legacy selector set.
2. Add a new file collection path for glob mode:
   - expand `--run` patterns
   - expand `--skip` patterns
   - recursively collect files under matched directories
   - deduplicate and sort the final file list
3. Keep the existing suite-based collection path for legacy mode.
4. Make `runner` consume a unified file list regardless of which mode produced it.
5. Add unit tests covering:
   - legacy directory-name lookup remains unchanged
   - glob mode includes files from file and directory patterns
   - glob mode excludes files from `--skip`
   - duplicate matches are deduplicated
6. Run formatting and targeted tests.

## Open Decisions Settled Here

- `--run` and `--skip` are filesystem globs relative to the current working directory unless an absolute path is provided.
- `--skip` is only valid together with `--run`.
- Legacy selectors remain available but are documented as compatibility options.
