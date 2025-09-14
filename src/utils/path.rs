/// Strips the Windows extended path prefix (\\?\) if present.
pub fn strip_windows_prefix(path: &std::path::PathBuf) -> std::path::PathBuf {
    #[cfg(windows)]
    {
        let s = path.to_string_lossy();
        if s.starts_with(r"\\?\") {
            use std::path::Path;
            return Path::new(&s[4..]).to_path_buf();
        }
    }
    path.clone()
}
use std::path::{Path, PathBuf};

pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            let home = strip_windows_prefix(&home);
            if path == Path::new("~") {
                return home;
            } else if let Ok(relative) = path.strip_prefix("~/") {
                return home.join(relative);
            }
        }
    }
    path.to_path_buf()
}

pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let expanded = expand_tilde(path);

    if let Ok(canonical) = expanded.canonicalize() {
        canonical
    } else {
        expanded
    }
}

pub fn is_hidden<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

pub fn get_parent_and_name<P: AsRef<Path>>(path: P) -> (PathBuf, String) {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    (parent, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/documents");
        if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home.join("documents"));
        }
    }

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(".hidden"));
        assert!(is_hidden("/path/to/.hidden"));
        assert!(!is_hidden("visible"));
        assert!(!is_hidden("/path/to/visible"));
    }

    #[test]
    fn test_get_parent_and_name() {
        let (parent, name) = get_parent_and_name("/path/to/file.txt");
        assert_eq!(parent, PathBuf::from("/path/to"));
        assert_eq!(name, "file.txt");

        let (parent, name) = get_parent_and_name("file.txt");
        assert_eq!(parent, PathBuf::from("."));
        assert_eq!(name, "file.txt");
    }
}