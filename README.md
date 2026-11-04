# `wasm-bindgen` table exhaustion demo

See [https://github.com/wasm-bindgen/wasm-bindgen/issues/5324#issuecomment-5788914049]

## Setup
Install `tauri`
1. [Prerequisites](https://v2.tauri.app/start/prerequisites/#macos)
2. [`tauri-cli`](https://v2.tauri.app/start/create-project/#start-the-development-server)

## Steps to reproduce
1. Run the `project-daemon`
```sh
cd local/project-daemon
cargo run -F server
```

2. Run the desktop app
```sh
cd desktop
cargo tauri dev
```
3. Open the inspector and go to the `Console` tab.
You can typically open the Inspector by pressing `F12` 
or right clicking and selecting it.

4. Create a user (data is only stored locally, can be fake info)

5. Delete the file `/users/thot/Library/Application Support/ai.syre.syre-local/local_config.json` then restore it a few times.
This show create the error.
