use crate::builder::build;
use bus::Bus;
use libdoctave::content_api::ViewMode;
use std::path::PathBuf;
use std::sync::{
    mpsc::{self, RecvTimeoutError},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum WatcherMessage {
    RebuildNeeded,
    WatchError(String),
}

#[derive(Debug, Clone)]
struct ReloadSignal;

pub struct DevArgs<'a, W: std::io::Write> {
    pub working_dir: PathBuf,
    pub port: Option<u16>,
    pub stdout: &'a mut W,
}

pub fn run<W: std::io::Write>(mut args: DevArgs<W>) -> crate::Result<()> {
    let port = args.port.unwrap_or(8080);
    let build_dir = args.working_dir.join("_build");

    // Build the project first
    writeln!(args.stdout, "Building project...")?;
    build(
        &mut args.stdout,
        &args.working_dir,
        &build_dir,
        ViewMode::Dev,
    )?;

    // Create watcher communication channel
    let (watcher_tx, watcher_rx) = mpsc::channel::<WatcherMessage>();

    // Create broadcast bus for reload signals
    let reload_bus = Arc::new(Mutex::new(Bus::<ReloadSignal>::new(10)));

    // Spawn HTTP server thread
    let http_build_dir = build_dir.clone();
    let http_reload_bus = reload_bus.clone();
    let http_handle =
        thread::spawn(move || spawn_http_server(http_build_dir, port, http_reload_bus));

    // Spawn file watcher thread
    let watcher_working_dir = args.working_dir.clone();
    let watcher_handle = thread::spawn(move || spawn_file_watcher(watcher_working_dir, watcher_tx));

    writeln!(
        args.stdout,
        "Dev server running on http://localhost:{}",
        port
    )?;
    writeln!(args.stdout, "Watching for file changes...")?;

    // Main coordination loop
    loop {
        match watcher_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(WatcherMessage::RebuildNeeded) => {
                writeln!(args.stdout, "Rebuilding...")?;

                match build(
                    &mut args.stdout,
                    &args.working_dir,
                    &build_dir,
                    ViewMode::Dev,
                ) {
                    Ok(_) => {
                        // Build function already prints "Build complete" message
                        // Send reload signal to all connected browsers
                        if let Ok(mut bus) = reload_bus.lock() {
                            bus.broadcast(ReloadSignal);
                        }
                    }
                    Err(e) => {
                        writeln!(args.stdout, "Build failed: {:?}", e)?;
                        // No reload signal on build failure
                    }
                }
            }
            Ok(WatcherMessage::WatchError(e)) => {
                writeln!(args.stdout, "Watch error: {}", e)?;
            }
            Err(RecvTimeoutError::Timeout) => {
                // Periodic health check every 5 seconds
                if http_handle.is_finished() {
                    return Err(crate::Error::General("HTTP server thread died".to_string()));
                }
                if watcher_handle.is_finished() {
                    return Err(crate::Error::General(
                        "File watcher thread died".to_string(),
                    ));
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err(crate::Error::General(
                    "File watcher disconnected".to_string(),
                ));
            }
        }
    }
}

fn spawn_http_server(
    build_dir: PathBuf,
    port: u16,
    reload_bus: Arc<Mutex<Bus<ReloadSignal>>>,
) -> Result<(), String> {
    let server = tiny_http::Server::http(format!("localhost:{}", port))
        .map_err(|e| format!("Failed to start server: {}", e))?;

    loop {
        let request = server
            .recv()
            .map_err(|e| format!("Failed to receive request: {}", e))?;

        match request.url() {
            "/dev-reload" => {
                // Create a new receiver for this SSE connection
                let reload_rx = match reload_bus.lock() {
                    Ok(mut bus) => bus.add_rx(),
                    Err(_) => return Err("Failed to lock reload bus".to_string()),
                };
                thread::spawn(move || {
                    handle_sse_connection(request, reload_rx);
                });
            }
            _ => {
                let response = handle_request(&request, &build_dir);
                let _ = request.respond(response);
            }
        }
    }
}

fn spawn_file_watcher(
    working_dir: PathBuf,
    watcher_tx: mpsc::Sender<WatcherMessage>,
) -> Result<(), String> {
    use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};

    let mut debouncer = new_debouncer(
        Duration::from_millis(150),
        move |res: DebounceEventResult| {
            match res {
                Ok(events) => {
                    // Filter events to only rebuild-worthy files
                    let should_rebuild = events
                        .iter()
                        .any(|event| should_rebuild_for_path(&event.path));

                    if should_rebuild {
                        // Just notify main thread that rebuild is needed
                        let _ = watcher_tx.send(WatcherMessage::RebuildNeeded);
                    }
                }
                Err(e) => {
                    let _ = watcher_tx
                        .send(WatcherMessage::WatchError(format!("Watch error: {:?}", e)));
                }
            }
        },
    )
    .map_err(|e| format!("Failed to create file watcher: {:?}", e))?;

    // Watch the working directory recursively
    debouncer
        .watcher()
        .watch(&working_dir, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to start watching: {:?}", e))?;

    // Keep the watcher alive
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn should_rebuild_for_path(path: &std::path::Path) -> bool {
    // Skip if path contains ignored directories
    if path.components().any(|component| {
        let name = component.as_os_str();
        name == "_build" || name == ".git" || name == "node_modules"
    }) {
        return false;
    }

    // Check file extension
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            return matches!(
                ext_str,
                "md" | "yaml" | "yml" | "json" | "png" | "jpg" | "jpeg" | "svg" | "css" | "js"
            );
        }
    }

    // Also watch config files without extensions or special names
    if let Some(file_name) = path.file_name() {
        if let Some(name_str) = file_name.to_str() {
            return matches!(name_str, "docapella.yaml" | "doctave.yaml");
        }
    }

    false
}

fn handle_sse_connection(request: tiny_http::Request, mut reload_rx: bus::BusReader<ReloadSignal>) {
    use std::io::Write;

    // Convert request to writer
    let mut writer = request.into_writer();

    // Send SSE headers
    let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
    if writer.write_all(headers.as_bytes()).is_err() {
        return;
    }

    // Send initial connection message
    if writer.write_all(b"data: connected\n\n").is_err() {
        return;
    }

    // Listen for reload signals
    loop {
        match reload_rx.recv() {
            Ok(_) => {
                // Send reload message to browser
                if writer.write_all(b"data: reload\n\n").is_err() {
                    break;
                }
                if writer.flush().is_err() {
                    break;
                }
            }
            Err(_) => {
                // Channel closed, exit
                break;
            }
        }
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
