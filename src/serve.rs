use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

/// State of the server that manages the http requests
pub struct AppState {
    base_path: PathBuf,
}

impl AppState {
    pub fn new(base_path: &PathBuf) -> Self {
        Self {
            base_path: base_path.to_owned(),
        }
    }

    pub fn base_path(&self) -> PathBuf {
        self.base_path.join("index.html")
    }

    pub fn page_path(&self, path: &str) -> PathBuf {
        let page = self.base_path.join(path);
        let index = page.clone().join("index.html");
        let html = page.clone().with_extension("html");
        if PathBuf::from(&index).exists() {
            index
        } else if PathBuf::from(&html).exists() {
            html
        } else {
            page
        }
    }
}

/// Handler to serve the homepage
///
/// Input:
/// - state: server's state
pub async fn homepage(state: web::Data<Arc<AppState>>) -> impl Responder {
    match fs::read(state.base_path()).await {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(_) => HttpResponse::InternalServerError().body("Could not read file"),
    }
}

/// Handler to read any html, and more
///
/// Input:
/// - state: server's state
/// - path: path to read and return
pub async fn pages(state: web::Data<Arc<AppState>>, path: web::Path<String>) -> impl Responder {
    match fs::read(state.page_path(&path)).await {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(e) => {
            println!("{:?}", e);
            println!("{:?}", state.page_path(&path));
            println!("{:?}", path);
            HttpResponse::InternalServerError().body("Page not found")
        }
    }
}

/// Serves the path indicated in the input at the address provided
///
/// Input:
/// - state: server's state
/// - path: path to read and return
pub async fn serve_directory(addr: (&str, u16), path: &PathBuf) {
    let shared_state = Arc::new(AppState::new(path));
    println!("serving {} on http://{}:{}", path.display(), addr.0, addr.1);
    HttpServer::new(move || {
        let state = shared_state.clone();
        App::new()
            .app_data(web::Data::new(state))
            .route("/", web::get().to(homepage))
            .route("/{path:.*}", web::get().to(pages))
    })
    .bind(addr)
    .unwrap()
    .run()
    .await
    .unwrap();
}
