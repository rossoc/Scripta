use clap::Parser;
use std::path::PathBuf;

/// Convert markdown notes into HTML
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Set source directory
    #[arg(short, default_value = "./")]
    pub src: PathBuf,

    /// Set output directory
    #[arg(short, default_value = "./_site")]
    pub out: PathBuf,

    /// Watch for file changes and rebuild automatically
    #[arg(long, default_value_t = false)]
    pub watch: bool,

    /// Serve output directory locally after building
    #[arg(long, default_value_t = false)]
    pub serve: bool,

    /// Port to run the server on
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Open today's note at the folder, if not existing it is created
    #[arg(short, long, default_value = "/")]
    pub write: PathBuf,
}
