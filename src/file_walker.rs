use crate::error::Error;
use std::fs::{copy, create_dir_all, read_dir, read_to_string};
use std::iter::once;
use std::path::PathBuf;

/// Gets the file_name of the path and returns it as string
pub fn file_name(dir: &PathBuf) -> String {
    match dir.file_name() {
        Some(name) => path_to_str(&name.into()),
        None => "".to_string(),
    }
}

/// Convert a path to string, very similar to debug
pub fn path_to_str(dir: &PathBuf) -> String {
    match dir.to_str() {
        Some(res) => res.to_string(),
        None => "".to_string(),
    }
}

/// Given a path, returns a boolean true if such file is to be included, false
/// otherwise.
pub fn should_include(path: &PathBuf) -> bool {
    let ignoring = [
        "charta",
        "layout",
        "target",
        "src",
        ".git",
        ".github",
        ".obsidian",
        "script",
        "_site",
        ".gitignore",
    ]
    .iter()
    .map(|e| e.to_string())
    .collect::<Vec<_>>();

    !ignoring.contains(&file_name(&path))
}

/// Given a directory, returns such directory and all
/// sub-directories recursively. Files returned by `ignore_files()` are ignored.
///
/// Input:
/// - source_dir: the source directory to start the recursion
pub fn dirs_walker(source_dir: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    let src = PathBuf::from(source_dir);
    src.canonicalize()?;
    let res = read_dir(source_dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.path().canonicalize().ok())
        .filter(|e| should_include(e))
        .filter_map(|e| dirs_walker(&e).ok())
        .flatten()
        .chain(once(src.canonicalize()?))
        .collect();
    Ok(res)
}

// Return the list of files contained in the source folder
pub fn files_walker(source_folder: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    let walker = read_dir(source_folder)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>();
    Ok(walker)
}

/// Given the name of a layout (template), it returns the content.
pub fn read_layout(name: &str) -> Result<String, Error> {
    let mut path = PathBuf::from("layout");
    path.canonicalize()?;
    path.push(name.to_string() + ".html");
    match read_to_string(path) {
        Ok(layout) => Ok(layout),
        Err(_) => Err(Error::MissingLayout(name.to_string())),
    }
}

/// Copy all the files in the source directory to the destination.
///
/// Input:
/// - src: source directory
/// - dst: destination directory
pub fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<(), std::io::Error> {
    if !should_include(src) {
        return Ok(());
    }

    if !dst.exists() {
        create_dir_all(dst)?;
    }

    for entry in read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
