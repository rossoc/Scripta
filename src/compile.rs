use crate::error::Error;
use crate::file_walker::read_layout;
use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use std::collections::HashMap;
use std::iter::once;

/// Given some Markdown it returns the equivalent html
///
/// Input:
/// - md: a reference to a string that contains the Markdown content
pub fn md_to_html(md: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(md, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Given the content of a Note (Markdown file as provided) returns a HashMap of
/// (key, value) stored in such Note
///
/// Input:
/// - content: the content of the Note
pub fn read_vars(content: &str) -> Result<HashMap<&str, String>, Error> {
    let mut pieces = content.splitn(3, "---");
    pieces.next();
    let settings = pieces.next().ok_or(Error::SettingsNotFoundGeneric)?;
    let content = pieces.next().ok_or(Error::ContentNotFoundGeneric)?.trim();

    let res = settings
        .lines()
        .filter_map(|s| s.split_once(':'))
        .map(|(k, v)| (k.trim(), v.trim().to_string()))
        .chain(once(("content", md_to_html(content))))
        .collect::<HashMap<&str, String>>();
    Ok(res)
}

/// Given the name of a layout, if it is found, it is built and returned.
///
/// Layout are built accessing the layout content with [`crate::file_walker::read_layout`].
/// A layout may includes others with the inclusion statement: `use <layout>`,
/// so they are composed with each other.
///
/// Input:
/// - template: the name of the layout to build
fn build_layout(template: &str) -> Result<String, Error> {
    let re = Regex::new(r"^\s*use\s+(\S+)").expect("compile::replace_use: regex ill defined");
    read_layout(template)?
        .lines()
        .map(|line| match re.captures(line) {
            Some(capture) => build_layout(&capture[1]),
            None => Ok(line.to_string()),
        })
        .collect::<Result<String, _>>()
}

/// Given a content and a map, go through the content, when it finds the pattern
/// `{{ <key> }}` and `map.contains_key(key)`, it replaces the placeholder
/// with `map(key)`; otherwise nothing happens.
///
/// Input:
/// - content: the content to look for the placeholders
/// - vars: the map made by `(key, value)`
fn replace_vars(content: &str, vars: &HashMap<&str, String>) -> String {
    let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_-]+)\s*\}\}")
        .expect("compile::replace_var: regex ill defined");
    re.replace_all(content, |cap: &regex::Captures| {
        let key = &cap[1];
        match vars.get(key) {
            Some(s) => s.to_string(),
            None => format!("{{{{ {key} }}}}"),
        }
    })
    .to_string()
}

/// Given a Note and its environment, its equivalent html is returned.
///
/// Input:
/// - note: the content of the Note
/// - env: a map of key values
///
/// A Note differs from a Markdown content, because it contains also some
/// setting, usually layout and title
pub fn to_html(note: &str, env: &HashMap<&str, String>) -> Result<String, Error> {
    let mut vars = env.clone();
    vars.extend(read_vars(note)?);
    let content = match vars.get("layout") {
        Some(layout) => build_layout(layout),
        None => Err(Error::MissingLayoutGeneric),
    }?;
    Ok(replace_vars(&content, &vars))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random_range;

    fn generate_random_alphanumeric_string(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

        let random_string: String = (0..length)
            .map(|_| {
                let idx = random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        random_string
    }

    #[test]
    fn test_read_vars() {
        let k1 = generate_random_alphanumeric_string(10);
        let v1 = generate_random_alphanumeric_string(10);
        let k2 = generate_random_alphanumeric_string(10);
        let v2 = generate_random_alphanumeric_string(10);
        let mut content = format!("---\n{k1}: {v1}\n{k2}: {v2}\n---\nHello World");

        let mut map = HashMap::new();
        map.insert(k1.as_str(), v1.clone());
        map.insert(k2.as_str(), v2.clone());
        map.insert("content", "<p>Hello World</p>\n".to_string());

        assert_eq!(read_vars(&content).unwrap(), map);

        content = format!("---\n{k1}: {v1}\n{k2}:{v2}\n---");
        map.insert("content", "".to_string());
        assert_eq!(read_vars(&content).unwrap(), map);

        content = format!("--\n{k1}: {v1}\n{k2}: {v2}\n---");
        let res = read_vars(&content);
        assert!(res.is_err());

        content = format!("---\n{k1}: {v1}\n{k2}: {v2}\n--");
        let res = read_vars(&content);
        assert!(res.is_err());

        content = format!("---\n\n---");
        map.clear();
        map.insert("content", "".to_string());
        assert_eq!(read_vars(&content).unwrap(), map);
    }

    #[test]
    fn test_replace_var() {
        let mut vars = HashMap::new();
        vars.insert("name", "Alice".to_string());
        vars.insert("score", "85".to_string());
        vars.insert("base-path", "hello".to_string());

        // Test: Normal replacement
        let content = "Hello {{ name }}, your score is {{ score }}!";
        let expected = "Hello Alice, your score is 85!";
        assert_eq!(replace_vars(content, &vars), expected);

        // Test: Normal replacement
        let content = "Hello {{ name }}, your pwd is {{ base-path }}!";
        let expected = "Hello Alice, your pwd is hello!";
        assert_eq!(replace_vars(content, &vars), expected);

        // Test: Missing variable should remain unchanged
        let content = "Hello {{ user }}, your score is {{ score }}!";
        let expected = "Hello {{ user }}, your score is 85!";
        assert_eq!(replace_vars(content, &vars), expected);

        // Test: Extra spaces in placeholders
        let content = "Hello {{  name  }}, score: {{  score   }}.";
        let expected = "Hello Alice, score: 85.";
        assert_eq!(replace_vars(content, &vars), expected);

        // Test: No placeholders
        let content = "No placeholders here.";
        let expected = "No placeholders here.";
        assert_eq!(replace_vars(content, &vars), expected);

        // Test: Empty string
        let content = "";
        let expected = "";
        assert_eq!(replace_vars(content, &vars), expected);

        // Test: Placeholder but empty map
        let vars: HashMap<&str, String> = HashMap::new();
        let content = "Hello {{ name }}!";
        let expected = "Hello {{ name }}!";
        assert_eq!(replace_vars(content, &vars), expected);
    }
}
