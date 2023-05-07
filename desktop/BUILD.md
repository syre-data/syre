# Build steps

Make sure to start at the root of the thot folder.

## Build database

- `cd local/database`

### On macOS and Linux

- `./build.sh`

> When building from m1 to intel: `./build_from_m1_to_intel.sh`

### On Windows

- `./build.bat`

## Add secrets to ENV

If you haven't already, go back to the root path `cd ../../`.

### On Bash / Zsh

- `set -o allexport && source .github/act/secrets && set +o allexport`

## Build application

- `cd desktop`
- `cargo tauri build`

> When building from m1 to intel: `cargo tauri build --target x86_64-apple-darwin`

## Build errors

`Error:error running bundle_dmg.sh`

Make sure no `Thot` process is running and that Thot is not mounted as a volume.
