# wheels-rs

Handwritten reference implementations of common libraries in Rust.

This is a personal project: building up a library ecosystem from scratch,
one small crate at a time. Each crate is meant to be small, focused, and
readable rather than maximally optimized. For real work, prefer the
standard library or established crates.

The taxonomy of "what libraries a mature ecosystem provides" is
maintained as a separate private knowledge base; this repo only holds
the Rust reference implementations.

The name `wheels` is a self-deprecating reference to "reinventing the
wheel". Other-language counterparts (if any) will live in separate
repos named `wheels-<lang-suffix>` (e.g. `wheels-ml` for OCaml).

## Crates

| Name           | Description                                |
|----------------|--------------------------------------------|
| `wheels-heap`  | Binary max-heap (priority queue)           |
| `wheels-vec`   | Dynamic array (growable vector)            |

## Layout

```
wheels-rs/
├── Cargo.toml          # workspace root
├── crates/
│   └── heap/           # wheels-heap crate
└── ...
```

## Building

```
cargo build
cargo test
```

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
