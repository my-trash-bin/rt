# rt

This repository contains a collection of Rust crates that together form a simple ray tracing renderer. The project is organized as a Cargo workspace.

## Workspace layout
- `rt` - command line application that loads scene definitions and renders images
- `core` - ray tracing core containing intersection logic and shading
- `scene` - scene description and serialization
- `json` - lightweight JSON parser used by the renderer
- `jsonc` - bindings to a C JSON-with-comments parser
- `json_minifier_cli` - CLI tool for minifying JSON files
- `pack` and `pack_cli` - utilities for building asset packs
- `types` - common math and color types

## Building

To build all crates in the workspace:

```sh
cargo build --workspace
```

## Running

The `rt` crate provides a CLI to render a scene file. A sample scene is included as `input.scene.rt`:

```sh
cargo run --package rt -- input.scene.rt
```

Scenes can also be loaded programmatically with `scene::Scene::from_json_value`:

```rust
let json = std::fs::read_to_string("input.scene.rt").unwrap();
let value = jsonc::parse(&json).unwrap();
let dummy_loader = MyLoader; // implements `scene::ImageLoader`
let scene = scene::Scene::from_json_value(value, 1.0, &dummy_loader).unwrap();
```

Use `--help` to see additional command line options such as output image dimensions and camera parameters.

## Formatting and tests

Before committing changes, ensure the code is formatted and that the workspace builds:

```sh
cargo fmt --all
cargo test --workspace
```
