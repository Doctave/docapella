use crate::file_gatherer::gather_files;
use libdoctave::{renderer::Renderer, ContentApiResponse, Project, ResponseContext};

use owo_colors::{OwoColorize as _, Stream};

use std::path::PathBuf;

pub struct BuildArgs<'a, W: std::io::Write> {
    pub working_dir: PathBuf,
    pub out_dir: PathBuf,
    pub stdout: &'a mut W,
}

pub fn run<W: std::io::Write>(args: BuildArgs<W>) -> crate::Result<()> {
    // Gather the files
    let files = gather_files(&args.working_dir)?;

    if files.is_empty() {
        return Err(crate::Error::General(format!(
            "No files found in directory: {}",
            args.working_dir.display()
        )));
    }

    let renderer = Renderer::new().expect("Failed to create renderer");

    match Project::from_file_list(files) {
        Ok(project) => {
            let start = std::time::Instant::now();

            for page in project.pages() {
                let mut path = args.out_dir.clone();
                path.push(page.out_path());

                if !path.exists() {
                    std::fs::create_dir_all(path.parent().unwrap())?;
                }

                let ctx = ResponseContext::default();
                let response = ContentApiResponse::content(page, &project, ctx);

                let rendered = renderer.render_page(response).map_err(|e| {
                    crate::Error::General(format!("Failed to render page: {:?}", e))
                })?;

                std::fs::write(path, rendered)?;
            }

            if !project.assets.is_empty() {
                for asset in &project.assets {
                    let path = args.out_dir.join(&asset.path);

                    if !path.exists() {
                        std::fs::create_dir_all(path.parent().unwrap())?;
                    }

                    std::fs::copy(
                        args.working_dir.join(&asset.path),
                        args.out_dir.join(&asset.path),
                    )?;
                }
            }

            let build_duration = start.elapsed();

            let start = std::time::Instant::now();

            let verify_results = project.verify(None, None);

            let verify_duration = start.elapsed();

            if let Err(e) = verify_results {
                writeln!(
                    args.stdout,
                    "Found {} issues while building documentation in {:?}",
                    e.len(),
                    verify_duration
                )?;

                for issue in e {
                    writeln!(
                        args.stdout,
                        "--------------------------------------------\n{} {}\n",
                        issue.message.bold(),
                        issue
                            .file
                            .map(|f| format!("[{}]", f.display()))
                            .unwrap_or(String::from(""))
                            .bold()
                    )?;
                    writeln!(args.stdout, "{}", issue.description)?;
                }

                writeln!(args.stdout, "--------------------------------------------",)?;
            }

            writeln!(
                args.stdout,
                "{} {}",
                "Build complete in".if_supports_color(Stream::Stdout, |s| s.green()),
                format!("{:?}", build_duration).if_supports_color(Stream::Stdout, |s| s.bold()),
            )?;

            Ok(())
        }
        Err(e) => {
            println!("{:?}", e);
            Err(crate::Error::General(String::from(
                "Failed to build project",
            )))
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

        fs::write(
            working_dir.path().join("doctave.yaml"),
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
