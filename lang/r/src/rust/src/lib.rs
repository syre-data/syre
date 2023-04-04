mod functions;
mod resources;

use extendr_api::prelude::*;

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod thot;
    use functions;
    use resources;
}
