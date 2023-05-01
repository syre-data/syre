# Build steps

Make sure to start at the root of the thot folder.

## Build database

- `cd local/database`

### On macOS and Linux

- `./build.sh`

### On Windows

- `./build.bat`

## Add secrets to ENV

If you haven't already, go back to the root path `cd ../../`.

### On Bash / Zsh

- `set -o allexport && .github/act/source secrets && set +o allexport`

## Build application

- `cd desktop`
- `cargo tauri build`
