# Passthrough, a Horizon native mod

Right-clicks pass through wall signs, wall banners, item frames and paintings to the block behind. Client only.

## Layout

- `mixins/passthrough/runtime.rs`: rules, geometry, state and tests. It must not name any game type, because `cargo test` compiles it alone.
- `mixins/passthrough/*.rs` (the rest): one hook function each, compiled inside Horizon as part of `crate::game`.
- `mixins/passthrough/horizon-mixin.toml`: hook points. `compatibility.json`: the Horizon commit and hook API it was tested on.
- `scripts/horizon-check.sh`: applies the mod to Horizon and compile-checks the game. CI runs it.

## Rules

- Use Horizon's named hooks and `native_*` helpers only. Never reach into the game's private fields or functions.
- No panics: a panic stops the game. No `unwrap` or `expect` outside tests.
- Every hook checks `native_on` before it does work. Keep the packet hook's `match` first: it runs for every packet.
- Module and helper files may not use inner attributes, `//!` comments, `mod` (except the `#[cfg(test)]` one) or `include!`.
- This is a clean-room implementation. Do not read, download or reference the source of any other click-through mod.

## Before pushing

Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `scripts/horizon-check.sh`.

## Version bumps

The version is in four places: `Cargo.toml`, `horizon-mixin.toml`, `VERSION` in `runtime.rs`, and a new `CHANGELOG.md` section. `scripts/check-version.sh` checks they agree. A release is a `v<version>` tag on `main`.

When Horizon changes a hook: fix the mod, set `horizon_revision` in both `compatibility.json` and `horizon-mixin.toml` to the commit it now builds on, and bump the patch version.
