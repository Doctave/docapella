use std::path::Path;

use libdoctave::{InputContent, InputFile, Project};
use walkdir::WalkDir;

fn gather_files(root: &Path) -> std::io::Result<Vec<InputFile>> {
    let mut files = vec![];

    for entry in WalkDir::new(root) {
        let entry = entry?;

        if entry.path().is_dir() {
            continue;
        } else {
            let content = std::fs::read_to_string(entry.path())?;

            files.push(InputFile {
                path: entry.path().strip_prefix(root).unwrap().to_path_buf(),
                content: InputContent::Text(content),
            });
        }
    }

    Ok(files)
}

fn main() {
    let dir = gather_files(&Path::new("examples").join("partials")).expect("Unable to read files");

    let project = Project::from_file_list(dir).unwrap();
    let page = project.get_page_by_fs_path(Path::new("README.md")).unwrap();

    println!("{:?}", page.ast(None));
}
