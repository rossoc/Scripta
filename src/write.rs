use std::io::{Error, Write};
use std::path::PathBuf;

const HEADER: &str = "---
title: {{ today }}
layout: page
author: Carlo Rosso
date: {{ date }}
---\n";

/// Given a directory, it opens the note with today's date as filename, using the default EDITOR
///
/// Input:
/// - path: path to directory
pub fn open_note(mut path: PathBuf) -> Result<(), Error> {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    path.push(date.to_owned() + ".md");
    if !path.exists() {
        let today = chrono::Local::now().format("%d/%m").to_string();
        let mut file = std::fs::File::create(&path)?;
        let content = HEADER
            .replace("{{ date }}", &date)
            .replace("{{ today }}", &today);
        file.write_all(content.as_bytes())?;
    }

    // open the file with the default editor
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
    let _ = std::process::Command::new(editor).arg(path).status()?;

    Ok(())
}
