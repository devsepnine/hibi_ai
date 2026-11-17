# Style and API naming

## Let the configured formatter handle layout

Use the [official Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/) for default formatting. Its defaults include four-space indentation, 100-character lines, and trailing commas in multiline lists. Repository overrides remain valid.

Inspect `rust-toolchain.toml` or `rust-toolchain`, `Cargo.toml`, and `rustfmt.toml` or `.rustfmt.toml` before choosing a formatting command. Use the pinned toolchain and existing style settings. A nightly documentation URL does not require a nightly compiler or a new style edition.

For a workspace, check formatting from its root:

```bash
cargo fmt --all -- --check
```

For a separate crate, use its manifest:

```bash
cargo fmt --manifest-path path/to/Cargo.toml -- --check
```

Use the repository's saved run configuration when available. Apply formatting only within the requested scope. Do not mix unrelated reformatting into a behavioral change. If the pinned formatter and current guide differ, record the mismatch and keep the configured toolchain unless the task includes changing it.

## Review API names for meaning

Use the [Rust API Guidelines naming chapter](https://rust-lang.github.io/api-guidelines/naming.html) for the named criteria below.

| Criterion | Check |
|---|---|
| `C-CASE` | Types, traits, variants: `UpperCamelCase`, such as `Uuid`. Functions, modules, locals: `snake_case`. Constants, statics: `SCREAMING_SNAKE_CASE`. |
| `C-CONV` | `as_`: cheap borrowed view. `to_`: conversion work or owned copy. `into_`: consumes a non-Copy value; cost varies. |
| `C-GETTER` | Prefer `name` and `name_mut`. Indexed `get` and established exceptions remain valid. |
| `C-ITER`, `C-ITER-TY` | Homogeneous collections: `iter`, `iter_mut`, `into_iter`; corresponding iterator types: `Iter`, `IterMut`, `IntoIter`. |
| `C-FEATURE` | Meaningful, additive feature names; avoid `use-`, `with-`, and negative names. |
| `C-WORD-ORDER` | Keep related names consistent, such as `ParseAddrError`. |

Check actual signatures and behavior, not prefixes alone. A `to_` method can validate a borrowed value without allocating. Mutable view names follow the returned type, such as `as_mut_slice`. Do not rename a legitimate `get` or change ownership merely to fit a naming pattern. Preserve public compatibility unless the task includes an API change. Formatter success does not prove naming correctness.
