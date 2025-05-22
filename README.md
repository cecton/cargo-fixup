# cargo-fixup

A lightweight Cargo plugin that lets you **patch crates during build time**
using a custom `rustc-wrapper`.

This tool is useful when you want to:
- Apply quick fixes or experimental changes to dependencies without forking
  them.
- Inject debug code or instrumentation temporarily.
- Automate source patching as part of your workflow.

---

## How It Works

`cargo-fixup` installs a `rustc-wrapper` shell script into your project's
`.cargo/` directory, and adds the following to `.cargo/config.toml`:

```toml
[build]
rustc-wrapper = "./.cargo/rustc-wrapper.sh"
```

At build time, this wrapper:

1. Intercepts the compilation of each dependency.
2. Checks if a patch exists under `patches/<crate-name>/`.
3. If it finds patch files, it:

   * Clones the original crate source to `target/patched-crates/<crate-name>/`
   * Applies the patches
   * Redirects the build to use the patched crate instead of the original

All patch logs are written to a file you specify during installation.

---

## Installation

```bash
cargo install cargo-fixup
cargo fixup /tmp/patch-log.txt
```

This writes the wrapper script and (overwrite) `config.toml` into your
**current workspace's** `.cargo/` directory.

**Note:** `cargo-fixup` only sets up the patching mechanism — it does not need
to be installed for the patches to be applied during build.

By default, patching logs are discarded. If you want to capture logs (e.g. to
debug patch application issues), run `cargo fixup /path/to/logfile` to
configure where logs should be written during builds.

---

## Directory Structure

```
your-project/
├── patches/
│   └── clap-4.5.38/
│       └── clap-fix.patch
├── .cargo/
│   ├── config.toml
│   └── rustc-wrapper.sh
```

**Note:** The patch directory name must match the **exact package name and
version**, formatted as `<name>-<version>` (e.g. `clap-4.5.38`). This ensures
patches are applied only to the correct version of a crate.

---

## Example: Patching `clap`

Let's say you want to inject a constant into the `clap` crate. Create this
patch file at:

```
patches/clap-4.5.38/clap-fix.patch
```

```diff
diff '--color=auto' -Naur a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs	2006-07-24 03:21:28.000000000 +0200
+++ b/src/lib.rs	2025-05-22 09:29:48.300874016 +0200
@@ -100,3 +100,5 @@
 pub mod _features;
 #[cfg(feature = "unstable-doc")]
 pub mod _tutorial;
+
+pub const HELLO: &str = "Hello from clap!";
```

Then build your project as usual:

```bash
cargo build
```

If `clap` (4.5.38) is being compiled, your patch will be applied automatically.

---

## Notes

* The wrapper only runs when `cargo` is building external dependencies.
* Only patches present in the `patches/<crate-name>-<version>/` directory are
  applied.
* This tool doesn't modify `Cargo.toml` or interfere with `[patch]` or
  `[replace]`.

---

## Why Not Use `[patch]`?

Cargo’s `[patch]` feature requires maintaining a fork of the entire crate.
`cargo-fixup` is ideal for:

* Local quick fixes
* Debugging
* Experiments
* CI workflows where you don’t want to push forks

---

## Compatibility

* Linux, macOS, WSL (Unix shell required)
* Windows support: **requires Bash** (e.g. via Git Bash or WSL)

If you need native Windows support, consider rewriting the wrapper as a Rust
binary.

---

## Uninstall

To revert, delete:

* `.cargo/config.toml`
* `.cargo/rustc-wrapper.sh`
* `patches/`

---

## License

MIT OR Apache-2.0

---

## Contributing

This project does not accept issues, pull requests, or other contributions.
Forks are welcome — feel free to use, modify, and build on it as needed.

---

## See Also

* [`mettke/cargo-patch`](https://github.com/mettke/cargo-patch)
  Automates unpacking a crate into your workspace and adds a `[patch]` section
  in `Cargo.toml` to override it.

  **Comparison:** Uses Cargo's built-in `[patch]`; requires manual edits or
  workspace management. No dynamic patching.

* [`mokeyish/cargo-patch-crate`](https://github.com/mokeyish/cargo-patch-crate)
  Generates and applies `[patch]` overrides using local crate copies based on
  your config.

  **Comparison:** Similar goals, but uses static patching via Cargo
  configuration instead of runtime patching.

**cargo-fixup** is unique in that it applies patches **dynamically at build
time**, using a `rustc-wrapper`. This avoids modifying `Cargo.toml` or
committing forked code, making it ideal for temporary or experimental changes.
