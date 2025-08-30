use crate::compile::to_html;
use crate::error::Error;
use crate::file_walker::{copy_dir_all, dirs_walker, file_name, files_walker, path_to_str};
use chrono::Local;
use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;

/// Given a directory and the source, strip the source and
/// change the name to match the output name.
///
/// In particular if the file is `readme.md` it is converted to `index.html`,
/// otherwise if the file is a Markdown only the extension is changed to `.html`.
/// The path returned is relative
fn compute_out(dir: &PathBuf, src: &PathBuf) -> Result<PathBuf, Error> {
    let mut res = dir.clone();
    if file_name(&res).to_lowercase() == "readme.md" {
        res.set_file_name("index.html")
    } else if res.extension().is_some_and(|ext| ext == "md") {
        res.set_extension("html");
    }
    match res.strip_prefix(src) {
        Ok(res) => Ok(res.into()),
        Err(e) => Err(Error::SplitPrefixUnreachable(e)),
    }
}

/// This function is used to set the placeholder `{{ base-path }}` in each
/// layout.
fn compute_base_path(dir: &PathBuf, src: &PathBuf) -> Result<String, Error> {
    match dir.strip_prefix(src) {
        Ok(res) => Ok(res.iter().fold("./".into(), |acc, _| acc + "../")),
        Err(e) => Err(Error::SplitPrefixUnreachable(e)),
    }
}

fn read_title(src: &PathBuf) -> Option<String> {
    let content = read_to_string(src).ok()?;
    match content.lines().find(|s| s.starts_with("title:")) {
        Some(title) => Some(title[6..].trim().to_string()),
        None => None,
    }
}

/// Format a link for the function [`make_links`]
fn make_link(dir: &(PathBuf, PathBuf)) -> Result<String, Error> {
    let title = match read_title(&dir.0) {
        Some(title) => title,
        None => file_name(&dir.1),
    };
    Ok(format!(
        "<li><a href=\"{}\">{}</a></li>\n",
        path_to_str(&dir.1),
        title
    ))
}

/// Generates the list of links of a collection formatted as an HTML list.
/// It's used to set the placeholder `{{ links }}`.
fn make_links(links: &Vec<(PathBuf, PathBuf)>) -> String {
    links
        .iter()
        .filter(|(from, to)| file_name(to) != "index.html" && file_name(from) != "")
        .filter_map(|dir| make_link(dir).ok())
        .rev()
        .fold(String::new(), |acc, link| acc + &link)
}

/// Generates the list of links of a collection formatted as an HTML list.
/// It's used to set the placeholder `{{ links.rev }}`.
fn make_links_rev(links: &Vec<(PathBuf, PathBuf)>) -> String {
    links
        .iter()
        .filter(|(from, to)| file_name(to) != "index.html" && file_name(from) != "")
        .filter_map(|dir| make_link(dir).ok())
        .fold(String::new(), |acc, link| acc + &link)
}

/// Given source and destination, and the variables, produces the output HTML
/// in the specified folder.
///
/// Input:
/// - dirs: a tuple (source, destination)
/// - vars: HashMap to replace the placeholders `{{ <key> }}`
pub fn make_page(dirs: (&PathBuf, PathBuf), vars: &HashMap<&str, String>) -> Result<(), Error> {
    let content = match read_to_string(&dirs.0) {
        Ok(c) => Ok(c),
        Err(_) => Err(Error::MissingFile(dirs.0.clone())),
    };
    match to_html(&content?, &vars) {
        Ok(content) => Ok(write(&dirs.1, content)?),
        Err(Error::MissingLayoutGeneric) => Err(Error::MissingLayout(path_to_str(dirs.0))),
        Err(Error::SettingsNotFoundGeneric) => Err(Error::SettingsNotFound(path_to_str(dirs.0))),
        Err(Error::ContentNotFoundGeneric) => Err(Error::ContentNotFound(path_to_str(dirs.0))),
        Err(e) => Err(e),
    }
}

/// Given some directories generates all the HTML in a collection.
/// A collection is made by all the Markdown in the same directory.
///
/// Input:
/// - dir: path to collection's directory in the source
/// - src: source dir of the site
/// - out: output dir or the site
pub fn make_collection(dir: &PathBuf, src: &PathBuf, out: &PathBuf) -> Result<(), Error> {
    let output_dir = compute_out(dir, src)?;
    if !output_dir.exists() {
        create_dir_all(&output_dir)?;
    }

    let mut dirs = files_walker(dir)?
        .into_iter()
        .filter(|file| file.extension().is_some_and(|ext| ext == "md"))
        .filter_map(|file| compute_out(&file, src).ok().map(|o| (file.clone(), o)))
        .collect::<Vec<_>>();

    dirs.sort_by(|(a, _), (b, _)| {
        file_name(b)
            .to_lowercase()
            .cmp(&file_name(a).to_lowercase())
    });

    dirs.push(("".into(), output_dir));

    let mut vars = HashMap::new();
    vars.insert("base-path", compute_base_path(dir, src)?);
    vars.insert("folder", file_name(dir));
    vars.insert("links", make_links(&dirs));
    vars.insert("links-rev", make_links_rev(&dirs));

    for i in 1..dirs.len() - 1 {
        vars.insert("next-page", path_to_str(&dirs[i - 1].1));
        vars.insert(
            "next-page-title",
            read_title(&dirs[i - 1].0).unwrap_or_default(),
        );
        vars.insert("previous-page", path_to_str(&dirs[i + 1].1));
        vars.insert(
            "previous-page-title",
            read_title(&dirs[i + 1].0).unwrap_or_default(),
        );
        make_page((&dirs[i].0, out.join(&dirs[i].1)), &vars)?;
    }

    if dirs.len() > 2 {
        vars.insert("next-page", path_to_str(&dirs[dirs.len() - 1].1));
        vars.insert(
            "next-page-title",
            read_title(&dirs[dirs.len() - 1].0).unwrap_or_default(),
        );
        vars.insert("previous-page", path_to_str(&dirs[1].1));
        vars.insert(
            "previous-page-title",
            read_title(&dirs[1].0).unwrap_or_default(),
        );
    }
    if dirs.len() > 1 {
        make_page((&dirs[0].0, out.join(&dirs[0].1)), &vars)?
    }
    Ok(())
}

/// Given the source and the output directory, generates the HTML.
pub fn make_site(src: &PathBuf, out: &PathBuf) -> Result<String, Error> {
    copy_dir_all(src, out)?;
    if !out.exists() {
        create_dir_all(out)?;
    }

    let src = &src.canonicalize()?;
    let out = &out.canonicalize()?;

    let dirs = dirs_walker(src)?;

    for dir in dirs {
        make_collection(&dir, src, out)?;
    }

    Ok(format!(
        "Compiled at {}",
        Local::now().format("%d/%m/%Y-%H:%M:%S")
    ))
}
