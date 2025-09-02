//! # Markdown to html
//!
//! This program converts a directory of Markdown files to HTML files.
//! The source directory is ignored, so you can have a `readme.md` and an
//! `index.html`. It recurses in every sub-directory.
//!
//! ```markdown
//! target
//! ├── folder1
//! │   ├── file1.md
//! │   ├── file2.md
//! │   └── folder2.md
//! │       ├── file3.md
//! │       ├── file4.md
//! │       └── ...
//! ├── layout
//! │   ├── some_layout.md
//! │   └── ...
//! ├── assets
//! │   ├── some_asset.jpg
//! │   └── ...
//! └── ...
//! ```
//!
//!
//!
//! Once the program is run, it will wait for an event to occur in the target
//! directory.
//!
//! ## Idee
//! - usare axum per creare un server che serve i file html (molto simile a actix-web)

mod cli;
mod compile;
mod error;
mod file_walker;
mod parser;
mod serve;
mod watcher;
mod write;
use crate::parser::make_site;
use crate::serve::serve_directory;
use crate::watcher::watch_dir;
use crate::write::open_note;
use clap::Parser;
use cli::Cli;
use std::path::PathBuf;
use tokio::signal;
use tokio::task::spawn;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    match cli {
        Cli::Write(args) => open_note(args.path),
        Cli::Build(mut args) => {
            std::env::set_current_dir(&args.src)?;
            args.out =
                pathdiff::diff_paths(&args.out, &args.src).expect("Cannot create relative path");
            args.src = PathBuf::from("./");

            match make_site(&args.src, &args.out) {
                Ok(time) => println!("{}", time),
                Err(e) => println!("{}", e),
            };

            let target = args.src.to_owned();
            let dest = args.out.to_owned();

            let compile_fn = move || {
                match make_site(&target, &dest) {
                    Ok(time) => println!("{}", time),
                    Err(e) => println!("{}", e),
                };
            };

            tokio::select! {
            _ = async {
                let mut res = spawn(async {});
                if args.watch {
                    res = spawn(async move {
                         watch_dir(&args.src, &compile_fn).unwrap();
                    });
                }

                if args.serve {
                    let addr = ("127.0.0.1", args.port);
                    let dest_cp = args.out.to_owned();
                    serve_directory(addr, &dest_cp).await;
                }

                let _ = res.await;
            } => {},
                _ = signal::ctrl_c() => {
                    println!("Received Ctrl+C, shutting down...");
                    std::process::exit(0)
                }
            };
            Ok(())
        }
    }
}
