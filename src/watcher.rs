use crate::file_walker::{dirs_walker, files_walker, should_include};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::path::PathBuf;
use std::time::Duration;

pub fn exec_on_event<F>(src: &PathBuf, f: &F) -> Result<()>
where
    F: Fn(Event) -> (),
{
    // Create a watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let config = Config::default().with_poll_interval(Duration::from_secs(2).into());
    let mut watcher: RecommendedWatcher = Watcher::new(tx, config)?;

    dirs_walker(src)
        .unwrap()
        .iter()
        .filter_map(|dir| files_walker(dir).ok())
        .flatten()
        .for_each(|p| watcher.watch(&p, RecursiveMode::NonRecursive).unwrap());

    println!("Watching directory: {:?}", src);

    // Listen for events
    for event in rx {
        match event {
            Ok(event) => {
                if event.paths.iter().any(|p| should_include(p))
                    && (event.kind.is_create() || event.kind.is_modify() || event.kind.is_remove())
                {
                    f(event)
                }
            }
            Err(e) => println!("{}", e),
        };
    }

    Ok(())
}
