# AGENTS.md

## Project Overview
This repository contains a Rust game engine. The documentation stack is split into:
1. API reference generated from Rust doc comments via `rustdoc`
2. Conceptual and architectural documentation written in Markdown and built with `mdBook`

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

## mdBook Structure
Documentation pages live in:
- `docs/book/src/getting-started/`
- `docs/book/src/concepts/`
- `docs/book/src/architecture/`
- `docs/book/src/guides/`
- `docs/book/src/internals/`

Keep pages focused and scoped.

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

## Expected Outputs
Good documentation tasks usually produce one or more of:
- updated Rust doc comments
- new or updated Markdown pages under `docs/book/src/`
- improved crate/module overviews
- a short summary of undocumented areas still remaining