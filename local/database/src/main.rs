//! Runs a [`Database`].
use thot_local_database::server::Database;

fn main() {
    let mut db = Database::new();
    db.listen_for_commands();
}
