use std::path::Path;

use libdoctave::{InputContent, InputFile, Project};
use walkdir::WalkDir;
fn gather_files(dir: &Path) -> std::io::Result<Vec<InputFile>> {
    let mut out = vec![];

    for entry in WalkDir::new(dir) {
        let entry = entry?;

        if entry.path().is_dir() {
            continue;
        } else {
            let content = std::fs::read_to_string(entry.path())?;

            out.push(InputFile {
                path: entry.path().to_path_buf(),
                content: InputContent::Text(content),
            });
        }
    }

    Ok(out)
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let dir = &Path::new(&args[1]);

    let files = gather_files(dir).map_err(|e| format!("{:?}", e))?;

    let project = Project::from_file_list(files).unwrap();

    println!("{:#?}", project.pages());

    Ok(())
}
