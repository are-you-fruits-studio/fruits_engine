---
title: API Reference
description: A first API reference page for the Fruits Engine Docusaurus preview.
---

# API Reference

This is a placeholder API reference page for the `fruits_engine` crate and related workspace crates.

In the final documentation flow, this section can be generated from Rust source and doc comments, then published alongside the conceptual docs.

## Workspace Crates

The root `fruits_engine` crate depends on engine subsystems such as:

- `fruits_app`
- `fruits_ecs`
- `fruits_math`
- `fruits_render`
- `fruits_collision`
- `fruits_asset_loading`
- `fruits_asset_storage`

## Example Entry

```rust
use fruits_engine::*;

fn main() {
    // Real examples should be generated from compilable samples or tests.
}
```

This page is intentionally minimal: it proves that API Reference can have its own route, sidebar, and navigation while sharing the same Docusaurus site.
