use crate::file_gatherer::gather_files;
use crate::Result;
use std::path::Path;

use libdoctave::{renderer::Renderer, ContentApiResponse, Project, ResponseContext};
use owo_colors::{OwoColorize as _, Stream};

/// Builds the project by finding all the files in the working directory and rendering them to
/// the output directory.
pub(crate) fn build<W: std::io::Write>(
    stdout: &mut W,
    working_dir: &Path,
    out_dir: &Path,
) -> Result<()> {
    // Gather the files
    let files = gather_files(working_dir)?;

    if files.is_empty() {
        return Err(crate::Error::General(format!(
            "No files found in directory: {}",
            working_dir.display()
        )));
    }

    let renderer = Renderer::new().expect("Failed to create renderer");

    match Project::from_file_list(files) {
        Ok(project) => {
            let start = std::time::Instant::now();
            writeln!(stdout, "Verifying project...")?;

            let verify_results = project.verify(None, None);

            let verify_duration = start.elapsed();

            if let Err(e) = verify_results {
                writeln!(
                    stdout,
                    "Found {} issues while building documentation in {:?}",
                    e.len(),
                    verify_duration
                )?;

                for issue in e {
                    writeln!(
                        stdout,
                        "--------------------------------------------\n{} {}\n",
                        issue.message.bold(),
                        issue
                            .file
                            .map(|f| format!("[{}]", f.display()))
                            .unwrap_or(String::from(""))
                            .bold()
                    )?;
                    writeln!(stdout, "{}", issue.description)?;
                }

                writeln!(stdout, "--------------------------------------------",)?;
            }

            let start = std::time::Instant::now();

            for page in project.pages() {
                println!("Page out path: {:?}", page.out_path());
            }

            for page in project.pages() {
                let mut path = out_dir.to_path_buf();
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
                    let path = out_dir.join(&asset.path);

                    if !path.exists() {
                        std::fs::create_dir_all(path.parent().unwrap())?;
                    }

                    if !asset.path.exists() {
                        // The OpenAPI spec might not exist, but is counted as an asset, so we'll just skip it
                        // in this case. We'll have an error in verify informing the user.
                        continue;
                    }

                    std::fs::copy(working_dir.join(&asset.path), out_dir.join(&asset.path))?;
                }
            }

            let build_duration = start.elapsed();

            writeln!(
                stdout,
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
