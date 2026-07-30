# syre desktop

## Running locally

1. Tauri looks for a sidecar binary called `syre-project-daemon`, if this does not exist 
build it from one of the `local/project_daemon` build scripts. These automatically copy and rename
the executable in the correct manner.

2. However, you probably want to run the `project_daemon` manually so you can monitor its activity.
The app checks weather an instance of the project daemon is running or not, and if not will launch
the sidecar.
```sh
cd local/project_daemon
cargo run -F server
```

3. Run the desktop app.

