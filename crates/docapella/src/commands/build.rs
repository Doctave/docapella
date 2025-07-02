use crate::file_gatherer::gather_files;
use libdoctave::Project;

use owo_colors::{OwoColorize as _, Stream};

use std::path::PathBuf;

pub struct BuildArgs<'a, W: std::io::Write> {
    pub working_dir: PathBuf,
    pub out_dir: PathBuf,
    pub stdout: &'a mut W,
}

pub fn run<'a, W: std::io::Write>(args: BuildArgs<'a, W>) -> crate::Result<()> {
    // Gather the files
    let files = gather_files(&args.working_dir)?;

    if files.is_empty() {
        return Err(crate::Error::General(format!(
            "No files found in directory: {}",
            args.working_dir.display()
        )));
    }

    match Project::from_file_list(files) {
        Ok(project) => {
            let start = std::time::Instant::now();

            for page in project.pages() {
                let mut path = args.out_dir.clone();
                path.push(page.out_path());

                if !path.exists() {
                    std::fs::create_dir_all(path.parent().unwrap())?;
                }

                println!("About to write {}", path.display());
                std::fs::write(path, format!("Fake content for {}", page.uri_path()))?;
            }

            let duration = start.elapsed();

            writeln!(
                args.stdout,
                "{} in {:?}",
                "Build complete".if_supports_color(Stream::Stdout, |s| s.green()),
                duration
            )?;

            Ok(())
        }
        Err(e) => {
            println!("{:?}", e);
            return Err(crate::Error::General(String::from(
                "Failed to build project",
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use temp_dir::TempDir;

    #[test]
    fn builds_a_project() {
        let working_dir = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();
        let mut fake_stdout = std::io::sink();

        fs::write(working_dir.path().join("doctave.yaml"), "---\ntitle: Hello World").unwrap();
        fs::write(working_dir.path().join("README.md"), "# Hello World").unwrap();

        let result = run(BuildArgs {
            working_dir: working_dir.path().to_path_buf(),
            out_dir: out_dir.path().to_path_buf(),
            stdout: &mut fake_stdout,
        });

        if let Err(err) = result {
            panic!("{}", err);
        }

        assert!(
            fs::metadata(out_dir.path().join("index.html")).is_ok(),
            "index.html not created"
        );
    }

    #[test]
    fn logs_that_the_project_was_built() {
        let working_dir = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();
        let mut fake_stdout = std::io::Cursor::new(Vec::new());

        fs::write(working_dir.path().join("README.md"), "# Hello World").unwrap();

        let result = run(BuildArgs {
            working_dir: working_dir.path().to_path_buf(),
            out_dir: out_dir.path().to_path_buf(),
            stdout: &mut fake_stdout,
        });

        if let Err(err) = result {
            panic!("{}", err);
        }

        let fake_stdout = String::from_utf8(fake_stdout.into_inner()).unwrap();

        assert!(
            fake_stdout.contains("Build complete"),
            "Built project not logged"
        );
    }
}
