# Contributing to purefetch

Thanks for your interest! purefetch aims to stay small, fast, and
**dependency-free**. Bug reports, ideas, and pull requests are all welcome.

## Ground rules

- **Zero dependencies.** No crates.io dependencies — the tool is `std` plus raw
  Linux syscalls. A PR that adds a dependency will be asked to drop it.
- **No panics, no blocking.** Detection modules read `/proc` and `/sys` and must
  return an empty `Vec` when data is missing rather than `unwrap()`-ing or
  spawning long-running work.
- **MSRV is 1.70** — don't use newer `std` APIs (a dedicated CI job builds and
  tests on 1.70; `Cargo.lock` stays in the v3 format so pre-1.78 cargo can read it).
- Run `cargo fmt` and `cargo clippy` before submitting; CI enforces both.

## Building & checking

```sh
cargo build --release
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

## Adding a distro logo

1. Create `assets/logos/<id>.txt`, where `<id>` matches the `/etc/os-release`
   `ID` (so auto-detection works). Format:

   ```
   COLORS: <sgr> <sgr> ...
   <ascii art, one row per line>
   ```

   The `COLORS:` line lists SGR parameter strings (e.g. `31 37`), one per color;
   in the art `$1`..`$9` switch to the Nth color, `$$` is a literal `$` (the
   fastfetch escape — fastfetch art files work verbatim), and any other `$` not
   followed by a digit 1-9 is also literal. Keep it ≤ 40 columns wide and
   ~16–20 rows, and preserve leading spaces.
2. Add `<id>` to the `ORDER` list in `examples/genlogos.rs` (with any aliases).
3. Regenerate: `cargo run --example genlogos` — it rewrites `src/logo.rs` and runs
   `rustfmt` on it. Don't edit `src/logo.rs` by hand.
4. Check it: `cargo run -- --logo <id>`.

## Adding an info module

Create `src/detect/<name>.rs` exposing `pub fn detect() -> crate::detect::Rows`,
then register it (label + function) in the `groups` table in `src/main.rs`. Use
`Row::val(...)` for a single value and return an empty `Vec` when unavailable.
`src/detect/cpu.rs` is the canonical example.

## Adding a CPU architecture

`src/sys.rs` issues raw syscalls per architecture. To add one, provide a
`syscall3` implementation (inline asm for that target's syscall convention) plus
the `SYS_STATFS` / `SYS_IOCTL` numbers, all behind `#[cfg(target_arch = "...")]`.
The `struct Statfs` layout is shared by LP64 targets; 32-bit targets would need
their own (`statfs64`) layout. Verify with `cargo check --target <triple>` and,
ideally, a run under `qemu-<arch>`.

## Commit & PR style

- Small, focused commits with a `<area>: <summary>` subject line.
- By submitting a pull request you agree that your contribution is dual-licensed
  under **MIT OR Apache-2.0**, per the terms in the README.
