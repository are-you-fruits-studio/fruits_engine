# AGENTS.md

## Project Overview
This repository contains a Rust game engine. The documentation site is built with Docusaurus and published through GitHub Pages.

The documentation stack is split into:
1. API reference generated from Rust doc comments as native rustdoc HTML via `cargo doc`
2. Conceptual, architectural, and guide documentation written manually as Markdown under the Docusaurus docs tree

Generated API reference files are build artifacts. Do not commit generated rustdoc HTML, Docusaurus build output, or legacy `cargo-doc-docusaurus` artifacts unless the repository policy changes explicitly.

## Documentation Goals
When working on documentation, prioritize:
1. Accuracy over completeness
2. Public API documentation over private implementation details
3. Architecture explanations over superficial summaries
4. Small, compilable examples over pseudo-code
5. Minimal churn in already reviewed docs

## Sources of Truth
When documenting behavior, prefer these sources in order:
1. Rust source code
2. Existing tests
3. Existing README and docs pages
4. Naming and module structure
Do not invent behavior that is not supported by code.

## What to Document
Focus documentation work on:
- public structs, enums, traits, functions
- crate-level overviews
- module-level overviews for major subsystems
- engine concepts and architecture
- how-to guides for common extension points

## What to Avoid
Do not:
- rewrite large parts of reviewed docs unless necessary
- document private helper functions unless they are architecturally important
- invent unsupported features
- claim guarantees that are not enforced by the code
- generate vague filler text

## Rust Doc Comment Rules
For public API items:
- add `///` doc comments
- explain purpose first
- document important invariants and constraints
- document parameters and return behavior when useful
- add short examples when practical

For modules and crates:
- add `//!` top-level docs when missing

Prefer concise docs. Avoid repetitive wording.

## Docusaurus Structure
The Docusaurus site lives in `docs/`.

Important paths:
- `docs/docs/` - manually written documentation pages
- `target/doc/` - generated native rustdoc HTML from `cargo doc`; do not commit
- `docs/scripts/generate-docs-api.ps1` - Windows/local API generation pipeline
- `docs/scripts/generate-docs-api.sh` - Linux/GitHub Actions API generation pipeline
- `docs/scripts/build-and-serve.ps1` - local full docs build and foreground server
- `docs/scripts/clean-docs-artifacts.ps1` - cleanup for generated docs artifacts
- `docs/docusaurus.config.js` - Docusaurus configuration
- `.github/workflows/docs.yml` - GitHub Pages build/deploy workflow

Manual docs should be placed under `docs/docs/` and linked through `docs/sidebars.js`. API reference content should be produced from Rust doc comments instead of hand-editing generated rustdoc files.

Keep pages focused and scoped.

## Documentation Build Workflow
For local verification on Windows, prefer:

```powershell
.\docs\scripts\build-and-serve.ps1
```

This script regenerates native rustdoc HTML, builds the Docusaurus static site, copies rustdoc verbatim into `docs/build/api-reference/`, and serves `docs/build` in the current terminal session.

The local server is `docs/scripts/serve-build.mjs`, a small static server that emulates GitHub Pages exactly (serves `*.html` verbatim under the `/fruits_engine/` baseUrl). It is used instead of `docusaurus serve`, which 301-redirects `*.html` URLs and breaks rustdoc's relative navigation. rustdoc HTML is self-contained with relative links, so no path rewriting is performed during the copy.

Useful options:
- `-SkipRustdoc` - reuse existing `target/doc` rustdoc HTML
- `-SkipNpmInstall` - reuse existing `docs/node_modules`
- `-NoServe` - build only, without starting a local server
- `-Port 3001` - use a different local server port

To clean generated docs artifacts:

```powershell
.\docs\scripts\clean-docs-artifacts.ps1
```

Use `-RemoveNodeModules` only when a full dependency cleanup is intentionally needed.

GitHub Pages is built by `.github/workflows/docs.yml`. The workflow runs the Linux API generation script through `bash`, installs Docusaurus dependencies with `npm ci`, builds the site, and deploys `docs/build`.

## Style
Write in clear technical English.
Use concrete terminology consistent with the codebase.
Prefer:
- "owns", "borrows", "registers", "schedules", "uploads", "dispatches"
Avoid vague phrases like:
- "handles stuff"
- "works with things"
- "manages everything"

## Examples
Prefer examples that are:
- minimal
- realistic
- aligned with the actual API
- easy to paste into tests or sample apps

## Validation
Before finalizing docs changes:
- ensure names and paths match the code
- ensure cross-references are not obviously broken
- ensure examples are syntactically plausible
- do not mention subsystems that do not exist in the repository
- for documentation pipeline changes, run `.\docs\scripts\build-and-serve.ps1 -SkipRustdoc -SkipNpmInstall -NoServe` when existing rustdoc output and npm dependencies are available
- if Rust public API docs changed, run the full docs build without `-SkipRustdoc` before considering the generated API reference validated

## Expected Outputs
Good documentation tasks usually produce one or more of:
- updated Rust doc comments
- new or updated Markdown pages under `docs/docs/`
- improved crate/module overviews
- documentation pipeline or Docusaurus configuration updates
- a short summary of undocumented areas still remaining
