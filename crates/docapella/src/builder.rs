use crate::file_gatherer::gather_files;
use crate::Result;
use std::path::Path;

use libdoctave::content_api::ViewMode;
use libdoctave::{renderer::Renderer, ContentApiResponse, Project, ResponseContext};
use owo_colors::{OwoColorize as _, Stream};
use rayon::prelude::*;

/// Builds the project by finding all the files in the working directory and rendering them to
/// the output directory.
pub(crate) fn build<W: std::io::Write>(
    stdout: &mut W,
    working_dir: &Path,
    out_dir: &Path,
    view_mode: ViewMode,
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

            let results: Vec<Result<()>> = project
                .pages()
                .par_iter()
                .map(|page| {
                    let mut path = out_dir.to_path_buf();
                    path.push(page.out_path());

                    if !path.exists() {
                        std::fs::create_dir_all(path.parent().unwrap())?;
                    }

                    let mut ctx = ResponseContext::default();
                    ctx.options.webbify_internal_urls = true;
                    ctx.view_mode = view_mode.clone();
                    let response = ContentApiResponse::content(page.clone(), &project, ctx);

                    let rendered = renderer.render_page(response).map_err(|e| {
                        crate::Error::General(format!("Failed to render page: {:?}", e))
                    })?;

                    std::fs::write(path, rendered)?;

                    Ok(())
                })
                .collect();

            let mut errors: Vec<crate::Error> = vec![];
            for result in results {
                if let Err(e) = result {
                    errors.push(e);
                }
            }

            if !errors.is_empty() {
                writeln!(
                    stdout,
                    "Failed to build project. Found {} errors.",
                    errors.len()
                )?;
                for error in errors {
                    writeln!(stdout, "{:?}", error)?;
                }

                return Err(crate::Error::General(String::from(
                    "Failed to build project",
                )));
            }

            // Copy assets
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

            // Generate the search index
            if let Ok(index) = project.search_index() {
                std::fs::create_dir_all(out_dir.join("_assets"))?;
                std::fs::write(out_dir.join("_assets/search.json"), index.to_json())?;
            } else {
                writeln!(
                    stdout,
                    "Failed to generate search index. This is not a fatal error, but you may not be able to search your project."
                )?;
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
        Err(e) => Err(crate::Error::FatalBuildError(e)),
    }
}
