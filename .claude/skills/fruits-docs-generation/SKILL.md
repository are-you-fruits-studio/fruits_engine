---
name: fruits-docs-generation
description: >-
  Writes high-value rustdoc for a single target crate/module in the fruits_engine workspace:
  a rich crate-level overview split into "How to use" and "How to maintain", plus sparse
  item docs that only add non-obvious information. Invoke as /fruits-docs-generation <target>.
disable-model-invocation: true
---

# fruits-docs-generation

Write rustdoc for **one explicitly chosen** part of the `fruits_engine` workspace. Run
incrementally — one crate/module/folder/file per invocation — so docs grow in reviewable
chunks. Never document the whole workspace in one run.

The argument after the command is the **target**: a crate name, a module path, a folder, or a
single file.

## Guiding principle (read this first)

**Documentation is a liability, not an asset.** Every line written must be maintained. The
goal is *not* coverage — it is to add what the code cannot already say for itself. A reviewer
rejected an earlier "document everything" pass with *"isn't it obvious?"* on almost every
comment. Do not repeat that.

The value lives in two places, in this priority order:

1. **The crate overview page** — the main deliverable. A human-written guide split into
   *How to use* and *How to maintain*, with real examples.
2. **A few item-level `///` comments** — only where an item has non-obvious behavior
   (panics, safety, units, invariants, edge cases) that a competent Rust dev would *not*
   infer from the signature and name.

Everything else gets **no comment**. Rustdoc already renders signatures, types, the module
tree, and navigation; do not restate them.

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
Key constraints: code is the source of truth (never invent behavior); clear technical English;
concrete verbs ("owns", "borrows", "registers", "schedules"); no filler.

## 3. The crate overview is the main deliverable

Write (or extend) a `//!` block at the top of the crate root (`lib.rs`). Do **not** open it
with a bullet list of the crate's types — rustdoc generates that navigation already. Instead,
split the page into two top-level sections:

```rust
//! # fruits_<name>
//!
//! One or two sentences on what problem this crate solves.
//!
//! # How to use
//!
//! For someone *using* the engine. Cover only the public API they touch directly — never
//! internals. Lead with examples of the most common use-cases (Unity-docs style: show real
//! code, not prose about code). Use the idiomatic entry points the engine actually expects.
//!
//! # How to maintain
//!
//! For someone *developing/maintaining* this crate. Explain how it works inside: the
//! architecture, the data flow, non-obvious implementation choices, invariants, and caveats
//! a maintainer must know before changing it. This is where understanding of the private
//! code belongs — written as prose here, not as `///` on private items.
```

Notes:
- **Examples first, and make them realistic.** Prefer the entry points a real user would call.
  (For example, in the collision crate a user normally pulls the module in via the engine's
  *default modules* registration rather than calling the crate's internal `add_*_module_to`
  directly — check the codebase for the idiomatic path before writing the example.)
- Large sub-modules of a major subsystem may get their own `//!` with the same two-section
  split, but only when the module is substantial enough to deserve its own page. Don't add
  empty section headers to tiny modules.

## 4. Item-level `///` docs — only when they earn their place

Apply this test to **every** candidate comment before writing it:

> Would a competent Rust developer, reading the item's name, signature, and types, already
> know this? If yes — **write nothing.**

### When you do write, format the description to fit the item kind

Keep the description **short** (about one sentence) and match its grammar to what the item *is*:

- **Functions / methods** → a **verb phrase** describing the action: *"Registers the collision
  subsystem…"*, *"Returns the smallest enclosing box…"*.
- **Structs / enums / traits / type aliases** → a **noun phrase** naming the concept or object,
  not an action: *"An endpoint of a range of keys."*, *"The resulting type after applying `&`."*.
- **Constants** → a short **noun phrase** for what the value is — nothing more (no usage advice,
  no pointers to other parts of the module).

Describe **what the item is / what task it solves**, never how it is implemented. Keep
implementation details and prose about *other* items out of the description.

If there is a characteristic worth remembering that is *not* part of the item's definition (a
gotcha, a convention, a side effect like "appends instead of replacing", a coordinate space),
put it on its own `NOTE:` line after the description instead of stuffing it into the sentence:

```rust
/// A collision shape attached to an entity.
///
/// NOTE: the shape is interpreted in the entity's local space.
pub struct ColliderComponent { /* … */ }
```

`# Safety` / `# Panics` / `# Errors` / `# Examples` keep their standard rustdoc headings (a
`NOTE:` line is for everything else). Order them after the description and any `NOTE:`.

**Write a `///` only when it adds non-obvious information**, such as:
- `# Safety` — **required** on every public `unsafe fn` (invariants the caller must uphold).
- `# Panics` — when and why it panics.
- `# Errors` — what an `Err` actually means (beyond "it failed").
- **Units, coordinate space, or sign conventions** not visible in the type
  (e.g. "radians", "local space", "half-extents").
- **Invariants / constraints** the type cannot express.
- **Surprising behavior or edge cases**, or the *why* behind a choice.

**Do not write** docs that merely restate the code. Concrete things the reviewer rejected as
"obvious" — avoid all of these:
- naming a variant after its type (`/// A sphere.` on `Sphere(CollisionSphere)`);
- `From`/`Into`/wrapper one-liners (`/// Wraps a point as Point.`);
- trivial getters / `is_empty` / `new`-from-fields (`/// Returns true if empty.`);
- field docs that echo the field name (`/// Center of the box.` on `center`);
- restating the signature (`/// Builds the resource from an iterator of shapes and entities.`).

**Scope each item's doc to that item's own responsibility.** Describe what *this* element is or
does — never the things that use, schedule, or call it:
- a **component** doc describes the data it carries, *not* the system that reads it;
- a **resource** doc describes the data and its invariants, *not* the system that fills it;
- a **system** doc describes the effect it has on the world, *not* who schedules it.

**Never document private or nested items.** Private items, private fields, and nested `fn`s do
not render in rustdoc — `///` on them is pure maintenance burden. Knowledge about internals
belongs in the crate's *How to maintain* section as prose.

**Intra-doc links:** when you do mention another item, link it with `` [`Type`] `` /
`[text](Self::method)` / `` [`crate::Item`] `` so the reference is clickable.

## 5. Examples

Examples are the highest-value content — favor adding more of them over describing obvious code.

- Show **common, realistic use-cases**, not toy snippets that mirror a signature.
- Prefer **runnable** doctests (they compile and run via `cargo test --doc`) so they can't rot.
- If a realistic example needs a running app or can't compile cheaply standalone, use
  ` ```no_run ` (still type-checked) or, only as a last resort, ` ```ignore ` / ` ```text `.
- Examples live on the crate page and on public items; doctests can only reference the public API.

## 6. Preserve developer comments

**Never delete a developer's comment.** Existing `//`, `///`, `//!`, block, and `// todo:`
text must survive. You may reformat a non-Rust style (e.g. C#/XML `/// <summary>...</summary>`)
into idiomatic Rust, or fold a note into a doc comment — **but keep the original wording** and
mark carried-over text with a `Developer note:` prefix. Never paraphrase away or drop the
developer's original meaning.

## 7. Verify

From the repo root, in PowerShell:

```powershell
$env:RUSTDOCFLAGS = "-D rustdoc::broken-intra-doc-links"
cargo +nightly doc -p <owning_crate> --no-deps
```

- Fix any errors or broken-link warnings and re-run until clean.
- If you added runnable examples, also run `cargo +nightly test --doc -p <owning_crate>`.

## 8. Report

Summarize:
- the crate/module overview you wrote (the two sections),
- which items received a `///` and the non-obvious reason each one earned it,
- **what you deliberately left undocumented** — under this skill that is the expected, correct
  outcome for most items, not a gap,
- the verification result (doc build / doc-test status).

Do **not** commit — only when the user explicitly asks.
