use std::io::Write;
use std::os::unix::fs::PermissionsExt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().peekable();
    let _command = args.next();
    args.next_if(|x| x.as_str() == "fixup");

    let log_path = args.next().unwrap_or_else(|| String::from("/dev/null"));

    let metadata = cargo_metadata::MetadataCommand::new().no_deps().exec()?;

    let workspace_root = metadata.workspace_root;

    let cargo_dir = workspace_root.join(".cargo");
    std::fs::create_dir_all(&cargo_dir)?;

    // Write config.toml
    let config_path = cargo_dir.join("config.toml");
    let config_contents = r#"[build]
rustc-wrapper = "./.cargo/rustc-wrapper.sh"
"#;
    std::fs::write(&config_path, config_contents)?;

    // Write rustc-wrapper.sh
    let wrapper_path = cargo_dir.join("rustc-wrapper.sh");
    let script = format!(
        r#"#!/bin/sh

# Redirect all output to log file except for the final exec'd command
LOG_FILE="{log_path}"
exec 3>&1 4>&2
exec >>"$LOG_FILE" 2>&1

set -eu

# --- Constants and Paths ---
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../" && pwd)
PATCHES_DIR="$SCRIPT_DIR/patches"
TARGET_PATCHED_DIR="$SCRIPT_DIR/target/patched-crates"

# --- No patching needed - run original args ---
if [ -z "${{CARGO_PKG_NAME:-}}" ] || [ -z "${{CARGO_MANIFEST_DIR:-}}" ]; then
  exec 1>&3 2>&4
  exec "$@"
fi

ORIGINAL_DIR_NAME=$(basename "$CARGO_MANIFEST_DIR")
PATCH_DIR="$PATCHES_DIR/$ORIGINAL_DIR_NAME"

# --- Check for matching patch directory ---
if [ -d "$PATCH_DIR" ]; then
  PATCHED_SRC="$TARGET_PATCHED_DIR/$ORIGINAL_DIR_NAME"

  echo "Applying patches to $CARGO_PKG_NAME..."

  mkdir -p "$TARGET_PATCHED_DIR"
  rm -rf -- "$PATCHED_SRC"
  cp -RL -- "$CARGO_MANIFEST_DIR" "$PATCHED_SRC"

  for PATCH_FILE in "$PATCH_DIR"/*; do
    [ -f "$PATCH_FILE" ] || continue
    if [ -x "$PATCH_FILE" ]; then
      echo "Executing: $PATCH_FILE"
      (cd "$PATCHED_SRC" && "$PATCH_FILE")
    elif [ "${{PATCH_FILE##*.}}" = "patch" ]; then
      echo "Applying patch: $PATCH_FILE"
      patch -s -p1 -d "$PATCHED_SRC" < "$PATCH_FILE"
    else
      echo "Not executable nor patch file: $PATCH_FILE"
    fi
  done

  new_args=()
  for arg in "$@"; do
    new_args+=("${{arg//$CARGO_MANIFEST_DIR/$PATCHED_SRC}}")
  done

  exec 1>&3 2>&4
  exec "${{new_args[@]}}"
else
  exec 1>&3 2>&4
  exec "$@"
fi
"#,
    );

    let mut file = std::fs::File::create(&wrapper_path)?;
    file.write_all(script.as_bytes())?;
    let mut perms = file.metadata()?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&wrapper_path, perms)?;

    println!("Patch wrapper configured in: {}", cargo_dir);
    Ok(())
}
