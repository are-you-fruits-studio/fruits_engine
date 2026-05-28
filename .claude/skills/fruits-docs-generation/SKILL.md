---
name: fruits-docs-generation
description: >-
  Generates rustdoc documentation (crate/module //! overviews and /// comments on all
  items, including private) for a single target crate, module, folder, or file in the
  fruits_engine workspace. Invoke as /fruits-docs-generation <target>.
disable-model-invocation: true
---

# fruits-docs-generation

Add rustdoc documentation to a **single, explicitly chosen** part of the `fruits_engine`
workspace. This skill is run incrementally — one crate/module/folder/file per invocation —
so the engine can be documented in reviewable chunks as it grows. Never document the whole
workspace in one run.

The argument after the command is the **target**: a crate name, a module path, a folder, or
a single file.

## 1. Resolve the target

Parse the target argument and determine the scope of `.rs` files:

- **Crate name** (e.g. `fruits_collision`) → every `.rs` file under `<crate>/src/`.
- **Folder / module path** (e.g. `fruits_ecs/src/behavior`) → every `.rs` file in that folder, recursively.
- **Single file** (e.g. `fruits_ecs/src/world.rs`) → just that file.

Then find the **owning crate**: walk up from the target to the nearest parent `Cargo.toml`
that is a package (not the workspace root). You need its package name for the verification step
(`cargo doc -p <name>`).

**If no target was provided, stop and ask the user for one.** Do not fall back to documenting
the entire workspace.

Announce the resolved scope (list of files + owning crate) before editing.

## 2. Read the project rules

Read [AGENTS.md](../../../AGENTS.md) and follow its sections: *Rust Doc Comment Rules*,
*Style*, *Examples*, *What to Avoid*, *Sources of Truth*.

Key constraints:
- **Source of truth is the code itself** (then tests, then existing docs). Never invent behavior,
  guarantees, or subsystems that the code does not support.
- Write in **clear technical English**. Use concrete verbs the codebase uses ("owns", "borrows",
  "registers", "schedules", "dispatches", "uploads"). No filler like "handles stuff" / "manages everything".
- Be concise. Avoid repetitive wording.

## 3. Document every item in scope

For each file in scope, add documentation for **all** items — **including private ones**.

> Note: this is a deliberate, user-requested deviation from AGENTS.md's "public over private"
> guidance. This skill documents private items too, by design.

**Module/crate overviews (`//!`):**
- At the top of a crate root (`lib.rs`) add a `//!` overview, starting with a `# <crate_name>`
  heading, summarizing what the crate provides and its main entry points.
- At the top of a module root (`mod.rs`, or aggregator files like `mod x; pub use x::*;`) add a
  `//!` overview of the module's responsibility.

**Item docs (`///`):** add a `///` comment to every:
- `fn` (free functions, methods in `impl` blocks, trait methods)
- `struct` and each of its **fields**
- `enum` and each of its **variants**
- `trait` and its associated items
- `mod` declarations (or use `//!` inside the module file instead)
- `const`, `static`, `type` aliases

Content order: **purpose first**, then important invariants/constraints, then parameters and
return behavior when useful. Keep it short.

**Do not touch items that already have a doc comment** (keep churn minimal), unless the existing
comment is clearly incomplete or wrong.

**Never delete developer comments.** Any existing comment text written by a developer (doc
comments `///`/`//!`, line comments `//`, block comments, `// todo:` notes, etc.) must be
preserved. You may:
- reformat a non-Rust comment style (e.g. C#/XML `/// <summary>...</summary>`,
  `/// <returns>...</returns>`) into idiomatic Rust doc comments, **but keep the original wording**;
- merge a developer note into your new doc comment.

When you carry over preserved developer text, mark it so the source is clear — prefix it with
`Developer note:` (e.g. `/// Developer note: will be used later; ~50% slower but gives contact points.`).
Do not silently drop, paraphrase away, or replace the developer's original meaning.

## 4. Markdown sections

rustdoc recognizes these by their heading. Add them where they apply:

- **`# Safety`** — required on **every `unsafe fn`**. Describe the invariants the caller must uphold.
  This crate has substantial `unsafe` (FFI, archetypes); treat these as high priority.
- **`# Errors`** — for functions returning `Result`: explain what an `Err` means and when it occurs.
- **`# Panics`** — for functions that can panic: state when and under what conditions.
- **`# Examples`** — **only on public items**, with compiling code. Examples become doc-tests
  (`cargo test --doc`) and can only reference the **public** API. For private items, either omit
  examples or use ` ```text ` / ` ```ignore ` fenced blocks so doc-tests don't break. Keep examples
  minimal, realistic, and aligned with the actual API (per AGENTS.md *Examples*).

**Intra-doc links** (clickable cross-references): use `` [`Type`] ``, `[text](Self::method)`,
`` [`module::Item`] `` to link between rustdoc pages. Prefer linking to types/functions you mention.

## 5. Verify

After editing, confirm the docs compile and intra-doc links resolve. From the repo root, in
PowerShell:

```powershell
$env:RUSTDOCFLAGS = "-D rustdoc::broken-intra-doc-links"
cargo +nightly doc -p <owning_crate> --no-deps
```

- If there are errors or broken-link warnings, fix them and re-run until clean.
- If you added `# Examples` with runnable code, also run:
  ```powershell
  cargo +nightly test --doc -p <owning_crate>
  ```
- Reset `$env:RUSTDOCFLAGS` afterwards if needed.

## 6. Report

Summarize for the user:
- which files were modified,
- roughly how many items were documented,
- anything in the target still left undocumented and why,
- the verification result (doc build / doc-test status).

Do **not** commit — only commit when the user explicitly asks.
