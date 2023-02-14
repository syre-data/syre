//! Runs a [`Runner`].
use std::process::Command;
use thot_local_database::Client as DbClient;
use thot_local_runner::Runner;

fn main() {
    if !DbClient::server_available() {
        let _server = Command::new("./assets/thot-local-database-x86_64-unknown-linux-gnu")
            .spawn()
            .expect("could not start database server");
    }

    let runner = Runner::new();
    // @todo
    // runner.run(root, max_tasks);
}
