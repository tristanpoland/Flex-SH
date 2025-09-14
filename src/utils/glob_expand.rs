use anyhow::Result;
use glob::glob;
use std::path::PathBuf;

pub fn expand_glob(pattern: &str) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();

    for entry in glob(pattern)? {
        match entry {
            Ok(path) => results.push(path),
            Err(e) => eprintln!("Error expanding glob pattern: {}", e),
        }
    }

    // If no matches found, return the original pattern
    if results.is_empty() {
        results.push(PathBuf::from(pattern));
    }

    Ok(results)
}

pub fn has_glob_chars(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?') || pattern.contains('[')
}

pub fn expand_args(args: &[String]) -> Result<Vec<String>> {
    let mut expanded_args = Vec::new();

    for arg in args {
        if has_glob_chars(arg) {
            let expanded = expand_glob(arg)?;
            for path in expanded {
                expanded_args.push(path.to_string_lossy().to_string());
            }
        } else {
            expanded_args.push(arg.clone());
        }
    }

    Ok(expanded_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_glob_chars() {
        assert!(has_glob_chars("*.txt"));
        assert!(has_glob_chars("file?.log"));
        assert!(has_glob_chars("file[0-9].txt"));
        assert!(!has_glob_chars("regular_file.txt"));
    }

    #[test]
    fn test_expand_args() {
        let args = vec![
            "regular_file.txt".to_string(),
            "*.nonexistent".to_string(), // Should return as-is if no matches
        ];

        let result = expand_args(&args).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "regular_file.txt");
        assert_eq!(result[1], "*.nonexistent");
    }
}