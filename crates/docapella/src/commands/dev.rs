use std::path::PathBuf;

use libdoctave::Result;

pub struct DevArgs {
    working_dir: PathBuf,
}

pub fn run(args: DevArgs) -> Result<()> {
    // Build the project

    // Start the dev server

    // Listen for changes to the project and rebuild

    // Handle ctrl-c

    Ok(())
}
