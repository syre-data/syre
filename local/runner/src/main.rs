//! Runs a [`Runner`].
use std::process::Command;
use syre_local_database::Client as DbClient;
use syre_local_runner::Runner;

fn main() {
    if !DbClient::server_available() {
        let _server = Command::new("./assets/syre-local-database-x86_64-unknown-linux-gnu")
            .spawn()
            .expect("could not start database server");
    }

    let runner = Runner::new();
    // TODO runner.run(root, max_tasks);
}
