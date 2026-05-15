use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use tiny_http::{Response, Server};

pub fn serve_directory(addr: (&str, u16), path: &Path) {
    let server = Server::http(format!("{}:{}", addr.0, addr.1)).unwrap();
    let base_path = path.to_path_buf();

    println!("serving {} on http://{}:{}", path.display(), addr.0, addr.1);

    for request in server.incoming_requests() {
        let url = request.url();
        let file_path = resolve_file_path(&base_path, url);

        let response = match File::open(&file_path) {
            Ok(mut file) => {
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).unwrap();
                let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
                Response::from_data(contents).with_header(
                    tiny_http::Header::from_bytes(
                        &b"Content-Type"[..],
                        mime.to_string().as_bytes(),
                    )
                    .unwrap(),
                )
            }
            Err(_) => Response::from_string("File not found").with_status_code(404),
        };

        request.respond(response).unwrap();
    }
}

fn resolve_file_path(base_path: &Path, url: &str) -> PathBuf {
    let mut path = base_path.to_path_buf();
    let clean_url = url.trim_start_matches('/');

    if clean_url.is_empty() {
        path.push("index.html");
    } else {
        path.push(clean_url);
        let index_path = path.join("index.html");
        let html_path = path.with_extension("html");

        if index_path.exists() {
            path = index_path;
        } else if html_path.exists() {
            path = html_path;
        }
    }
    path
}
