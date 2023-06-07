# Build steps

Make sure to start at the root of the thot folder.

## macOS preliminary

### Certificates

When building from macOS, it is necessary to have the certificates (these can be found in the drive
currently under `dev/certificates`).

For this you need the following two files:

- `Developer ID Application: Brian Carlsen (63BA6GV3UX)`
- `Developer ID Certification Authority`

Simply drag and drop these files to the `Keychain Access` app in the login tab.

### XCode

Make sure XCode is installed, with it comes `altool`, you need to have this in your path!

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

> When building from m1 to intel: `cargo tauri build --target x86_64-apple-darwin --debug`

## Build errors

`Error:error running bundle_dmg.sh`

Make sure no `Thot` process is running and that Thot is not mounted as a volume.
