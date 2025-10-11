use crate::builder::build;
use libdoctave::content_api::ViewMode;

use std::path::PathBuf;

pub struct BuildArgs<'a, W: std::io::Write> {
    pub working_dir: PathBuf,
    pub out_dir: PathBuf,
    pub stdout: &'a mut W,
}

pub fn run<W: std::io::Write>(mut args: BuildArgs<W>) -> crate::Result<()> {
    build(
        &mut args.stdout,
        &args.working_dir,
        &args.out_dir,
        ViewMode::Prod,
    )
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

        fs::write(
            working_dir.path().join("docapella.yaml"),
            "---\ntitle: Hello World",
        )
        .unwrap();
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

        fs::write(
            working_dir.path().join("docapella.yaml"),
            "---\ntitle: Hello World",
        )
        .unwrap();
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

    #[test]
    fn cleans_up_the_build_directory() {
        let working_dir = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();
        let mut fake_stdout = std::io::Cursor::new(Vec::new());

        fs::write(
            working_dir.path().join("docapella.yaml"),
            "---\ntitle: Hello World",
        )
        .unwrap();
        fs::write(working_dir.path().join("README.md"), "# Hello World").unwrap();

        fs::write(out_dir.path().join("foo.txt"), "").unwrap();

        let result = run(BuildArgs {
            working_dir: working_dir.path().to_path_buf(),
            out_dir: out_dir.path().to_path_buf(),
            stdout: &mut fake_stdout,
        });

        assert!(result.is_ok());

        // Check that foo.txt was removed
        assert!(!working_dir.path().join("_build/foo.txt").exists());
    }
}
