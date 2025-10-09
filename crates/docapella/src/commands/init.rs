use crate::Result;
use libdoctave::Project;
use owo_colors::OwoColorize as _;

use std::path::Path;

pub struct InitArgs<'a, W: std::io::Write> {
    pub working_dir: &'a Path,
    pub title: Option<&'a str>,
    pub stdout: &'a mut W,
}

pub fn run<W: std::io::Write>(args: InitArgs<W>) -> Result<()> {
    write!(args.stdout, "Creating project...")?;

    for page in Project::boilerplate_file_list() {
        let path = args.working_dir.join(page.0);
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, page.1)?;
    }

    writeln!(
        args.stdout,
        "{}\nRun {} to preview your documentation",
        "Done ✓".green(),
        "`docapella dev`".bold().blue()
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;
    use temp_dir::TempDir;

    #[test]
    fn creates_a_docapella_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();
        let mut fake_stdout = std::io::sink();

        let result = run(InitArgs {
            working_dir,
            title: Some("Docapella Starter Template"),
            stdout: &mut fake_stdout,
        });

        let docapella_yaml_path = working_dir.join("docapella.yaml");
        let contents = read_to_string(docapella_yaml_path).unwrap();

        assert!(contents.contains("title: Docapella Starter Template"));
        assert!(result.is_ok());
    }

    #[test]
    fn creates_a_readme() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();
        let mut fake_stdout = std::io::sink();

        let result = run(InitArgs {
            working_dir,
            title: Some("My Project"),
            stdout: &mut fake_stdout,
        });

        let readme_path = working_dir.join("README.md");
        let contents = read_to_string(readme_path).unwrap();

        assert!(
            contents.contains("Docapella Starter Template"),
            "Could not find welcome message in README.md: {}",
            contents
        );

        assert!(result.is_ok());
    }

    #[test]
    fn logs_out_progress_to_stdout() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();
        let mut fake_stdout = std::io::Cursor::new(Vec::new());

        let result = run(InitArgs {
            working_dir,
            title: Some("My Project"),
            stdout: &mut fake_stdout,
        });

        assert!(result.is_ok());

        let output = String::from_utf8(fake_stdout.into_inner()).unwrap();

        assert!(output.contains("Creating project..."));
        assert!(output.contains("Done!"));
    }
}
