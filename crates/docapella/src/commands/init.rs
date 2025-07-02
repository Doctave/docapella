use crate::Result;
use owo_colors::{OwoColorize as _, Stream};

use indoc::formatdoc;

use std::path::Path;

pub struct InitArgs<'a, W: std::io::Write> {
    pub working_dir: &'a Path,
    pub title: Option<&'a str>,
    pub stdout: &'a mut W,
}

pub fn run<W: std::io::Write>(args: InitArgs<W>) -> Result<()> {
    let docapella_yaml_path = args.working_dir.join("docapella.yaml");

    write!(args.stdout, "Creating docapella.yaml...")?;
    let contents = formatdoc!(
        r#"
        ---
        title: {}
        "#,
        args.title.unwrap_or("My docs project"),
    );
    writeln!(
        args.stdout,
        "{}",
        "✓".if_supports_color(Stream::Stdout, |s| s.green())
    )?;

    std::fs::write(docapella_yaml_path, contents)?;

    write!(args.stdout, "Creating README.md...")?;
    let readme_path = args.working_dir.join("README.md");
    let contents = formatdoc!(
        r#"
        # {}

        Welcome to your new Docapella project!

        ## Getting Started

        To get started, run the following command in your terminal:

        ```bash
        docapella dev
        ```

        This will start a local server and open your documentation in your browser.

        ## Documentation

        You can find the documentation for your project at the following URL:

        [http://localhost:8080](http://localhost:8080)
        "#,
        args.title.unwrap_or("My docs project")
    );
    writeln!(
        args.stdout,
        "{}",
        "✓".if_supports_color(Stream::Stdout, |s| s.green())
    )?;

    std::fs::write(readme_path, contents)?;

    writeln!(
        args.stdout,
        "Done! Run `docapella dev` to preview your documentation",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::fs::read_to_string;
    use temp_dir::TempDir;

    #[test]
    fn creates_a_docapella_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path();
        let mut fake_stdout = std::io::sink();

        let result = run(InitArgs {
            working_dir,
            title: Some("My Project"),
            stdout: &mut fake_stdout,
        });

        let docapella_yaml_path = working_dir.join("docapella.yaml");
        let contents = read_to_string(docapella_yaml_path).unwrap();

        assert_eq!(
            contents,
            indoc!(
                r#"
            ---
            title: My Project
            "#
            )
        );

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
            contents.contains("Welcome to your new Docapella project!"),
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

        assert!(output.contains("Creating docapella.yaml"));
        assert!(output.contains("Creating README.md"));
        assert!(output.contains("Done!"));
    }
}
