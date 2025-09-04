use crate::builder::build;
use std::path::PathBuf;

pub struct DevArgs<'a, W: std::io::Write> {
    pub working_dir: PathBuf,
    pub port: Option<u16>,
    pub stdout: &'a mut W,
}

pub fn run<W: std::io::Write>(args: DevArgs<W>) -> crate::Result<()> {
    let port = args.port.unwrap_or(8080);
    let build_dir = args.working_dir.join("_build");

    // Build the project first
    writeln!(args.stdout, "Building project...")?;
    build(args.stdout, &args.working_dir, &build_dir)?;

    // Start the HTTP server
    writeln!(
        args.stdout,
        "Starting dev server on http://localhost:{}",
        port
    )?;

    let server = tiny_http::Server::http(format!("localhost:{}", port))
        .map_err(|e| crate::Error::General(format!("Failed to start server: {}", e)))?;

    loop {
        let request = server
            .recv()
            .map_err(|e| crate::Error::General(format!("Failed to receive request: {}", e)))?;

        let response = handle_request(&request, &build_dir);
        let _ = request.respond(response);
    }
}

fn handle_request(
    request: &tiny_http::Request,
    build_dir: &std::path::Path,
) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    let url = request.url();
    let path = resolve_path(url, build_dir);

    match std::fs::read(&path) {
        Ok(content) => {
            let content_type = content_type_for_path(&path);
            tiny_http::Response::from_data(content).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                    .expect("Invalid content type header"),
            )
        }
        Err(_) => {
            let not_found = b"404 Not Found";
            tiny_http::Response::from_data(not_found.to_vec())
                .with_status_code(404)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..])
                        .expect("Invalid content type header"),
                )
        }
    }
}

fn resolve_path(url: &str, build_dir: &std::path::Path) -> PathBuf {
    let clean_url = url.trim_start_matches('/');

    if clean_url.is_empty() {
        // Root path, try index.html
        return build_dir.join("index.html");
    }

    // Try direct path first
    let direct_path = build_dir.join(clean_url);
    if direct_path.exists() {
        if direct_path.is_dir() {
            // If it's a directory, try index.html inside
            let index_path = direct_path.join("index.html");
            if index_path.exists() {
                return index_path;
            }
        } else {
            return direct_path;
        }
    }

    // If direct path doesn't exist, try adding .html extension
    let html_path = build_dir.join(format!("{}.html", clean_url));
    if html_path.exists() {
        return html_path;
    }

    // Fall back to the original direct path (will result in 404)
    direct_path
}

fn content_type_for_path(path: &std::path::Path) -> String {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    match extension {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        _ => "text/plain; charset=utf-8",
    }
    .to_string()
}
