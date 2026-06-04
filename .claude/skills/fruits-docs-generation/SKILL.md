---
name: fruits-docs-generation
description: >-
  Writes high-value rustdoc for a single target crate/module in the fruits_engine workspace.
  All documentation lives on the crate main page (a //! overview split into "How to use" and
  "How to maintain"); item-level /// comments are not used. Invoke as /fruits-docs-generation <target>.
disable-model-invocation: true
---

# fruits-docs-generation

Write rustdoc for **one explicitly chosen** part of the `fruits_engine` workspace. Run
incrementally — one crate/module/folder/file per invocation — so docs grow in reviewable
chunks. Never document the whole workspace in one run.

The argument after the command is the **target**: a crate name, a module path, a folder, or a
single file.

## Guiding principle (read this first)

**The code explains itself; the main page explains the crate.** Item-level doc comments are a
liability — they restate what the signature already says and rot the moment the code changes. A
reviewer rejected a "document everything" pass with *"isn't it obvious?"* on almost every line.

So this skill produces **exactly one kind of documentation**: the crate-level `//!` overview on
the main page. That is the whole deliverable.

- **Do not write item-level `///` comments** on functions, structs, enums, traits, constants,
  type aliases, fields, variants, or methods — public or private.
- **Remove existing `///` item docs** in the target scope (the knowledge they carried, if it is
  genuinely non-obvious, moves into the *How to maintain* section as prose). This does **not**
  apply to plain developer comments — see §4.
- Let names, types, and signatures carry the per-item meaning. If an item is so unclear that it
  *needs* a sentence, that is a signal to rename it, not to comment it.

## 1. Resolve the target

Determine the scope of `.rs` files:

- **Crate name** (e.g. `fruits_collision`) → the whole crate under `<crate>/src/`.
- **Folder / module path** (e.g. `fruits_ecs/src/behavior`) → that folder, recursively.
- **Single file** (e.g. `fruits_ecs/src/world.rs`) → just that file.

Find the **owning crate** (nearest parent package `Cargo.toml`) for the verification step.

If no target was provided, **stop and ask**. Do not document the whole workspace.

Announce the resolved scope (files + owning crate) before editing.

## 2. Read the project rules

Read [AGENTS.md](../../../AGENTS.md): *Sources of Truth*, *Style*, *Examples*, *What to Avoid*.
Key constraints: **code is the source of truth — never invent behavior**; clear technical
English; concrete verbs ("owns", "borrows", "registers", "schedules"); no filler.

## 3. The crate overview is the only deliverable

Write (or extend) a `//!` block at the top of the crate root (`lib.rs`). Do **not** open it with
a bullet list of the crate's types — rustdoc generates that navigation already. The page has
three parts: a one- or two-sentence summary, then two top-level sections.

```rust
//! # fruits_<name>
//!
//! One or two sentences on what problem this crate solves.
//!
//! # How to use
//!
//! For someone *using* the engine. Public API only, never internals.
//!
//! # How to maintain
//!
//! For someone *developing/maintaining* this crate: how it works inside.
```

### Both sections must be structured, not a wall of text

Break each section into small sub-sections, one per topic or use-case. Use a **deep heading
level** for them (`####`, i.e. h4) so they read as quiet sub-headings — rustdoc renders `#`/`##`
large and underlined, which would compete with the `How to use` / `How to maintain` section
titles; `####` is smaller, dimmer, and rule-less. Every worked example is a self-contained unit
with three clearly separated parts:

1. a `####` **heading** naming the use-case ("Making an entity collidable"),
2. a **code block** (the example),
3. **prose** above or below it explaining what the code does.

Never run two unrelated examples together without a heading between them. Order *How to use* by
how common the use-case is — most common first (Unity-docs style: show real code, lead with it).

### How to use

- Cover only the **public API a user touches directly**. Use the idiomatic entry points the
  engine actually expects — check the codebase for the real path (e.g. a subsystem is usually
  pulled in via the engine's *default modules* registration, not by calling a crate's internal
  `add_*_module_to` directly).
- Lead with examples. Prefer **runnable** doctests (compiled by `cargo test --doc`) so they
  cannot rot. If a realistic example needs a running app or can't compile cheaply standalone, use
  ` ```no_run ` (still type-checked) or, as a last resort, ` ```ignore ` / ` ```text `. Doctests
  can only reference the crate's public API.

### How to maintain

- Explain the architecture, data flow, the non-obvious implementation choices, invariants, and
  the caveats a maintainer must know before changing the code. This is where understanding of the
  **private** code belongs — as prose here, never as `///` on private items.

### Accuracy: every word must match the code

This is the rule that gets violated most. **Describe only what the code actually does — do not
invent mechanisms, names, or behavior.**

- Trace the real control flow before writing. If you cannot point to the line that does a thing,
  do not claim it happens.
- Do not introduce vocabulary the code does not justify. A past draft wrote that a shape is
  *"re-indexed"* each frame — there is no indexing step in the code; the system simply rebuilds
  the resource from scratch. Use the term that matches what the code literally does ("rebuilt",
  "transformed", "inserted"), not an invented one.
- Prefer naming the concrete function/type that performs a step (with an intra-doc link) over a
  vague description, so the claim stays checkable.

**Intra-doc links:** link items you mention with `` [`Type`] `` / `[text](Self::method)` /
`` [`crate::Item`] `` so the reference is clickable. This is the *only* place links are written
(there are no item docs to link from).

## 4. Preserve developer comments

**Never delete a developer's comment.** Plain code comments — `//`, `// todo:`, block comments —
stay exactly where they are; they are notes to maintainers, not rustdoc, and this skill leaves
them untouched. (Removing `///` *item docs* per §3 is different and expected.)

If you meet a non-Rust doc style (e.g. C#/XML `/// <summary>...</summary>`) on an item whose
`///` you are removing, do not silently drop its wording: move the original text into the *How to
maintain* section (or keep it as a plain `//` comment) and mark carried-over text with a
`Developer note:` prefix. Never paraphrase away the developer's original meaning.

## 5. Verify

From the repo root, in PowerShell:

```powershell
$env:RUSTDOCFLAGS = "-D rustdoc::broken-intra-doc-links"
cargo +nightly doc -p <owning_crate> --no-deps
```

- Fix any errors or broken-link warnings and re-run until clean.
- If the overview contains runnable examples, also run `cargo +nightly test --doc -p <owning_crate>`.

## 6. Report

Summarize:
- the crate/module overview you wrote (its sub-sections and examples),
- which item-level `///` docs you removed,
- any developer comments you preserved or carried over,
- the verification result (doc build / doc-test status).

Do **not** commit — only when the user explicitly asks.
