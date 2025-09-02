use crate::file_walker::{dirs_walker, should_include};
use notify::{Config, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub fn watch_dir<F>(src: &PathBuf, f: &F) -> Result<()>
where
    F: Fn() -> (),
{
    // Create a watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let config = Config::default().with_poll_interval(Duration::from_secs(2).into());
    let mut watcher: RecommendedWatcher = Watcher::new(tx, config)?;

    dirs_walker(src)
        .unwrap()
        .iter()
        .for_each(|p| watcher.watch(&p, RecursiveMode::NonRecursive).unwrap());

    println!("Watching directory: {:?}", src);
    let debounce_duration = Duration::from_millis(100);
    let mut last_trigged = Instant::now();

    // Listen for events
    for event in rx {
        match event {
            Ok(event) => {
                if event.paths.iter().any(|p| should_include(p))
                    && (event.kind.is_create() || event.kind.is_modify() || event.kind.is_remove())
                    && Instant::now().duration_since(last_trigged) > debounce_duration
                {
                    f();
                    last_trigged = Instant::now();
                }
            }
            Err(e) => println!("{}", e),
        };
    }

    Ok(())
}
