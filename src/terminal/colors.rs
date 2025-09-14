use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub name: String,
    pub colors: HashMap<String, String>,
}

impl ColorScheme {
    pub fn default_scheme() -> Self {
        let mut colors = HashMap::new();
        colors.insert("command".to_string(), "blue".to_string());
        colors.insert("argument".to_string(), "white".to_string());
        colors.insert("error".to_string(), "red".to_string());
        colors.insert("success".to_string(), "green".to_string());
        colors.insert("info".to_string(), "cyan".to_string());
        colors.insert("warning".to_string(), "yellow".to_string());
        colors.insert("path".to_string(), "cyan".to_string());
        colors.insert("number".to_string(), "magenta".to_string());
        colors.insert("prompt_user".to_string(), "green".to_string());
        colors.insert("prompt_host".to_string(), "blue".to_string());
        colors.insert("prompt_path".to_string(), "yellow".to_string());
        colors.insert("prompt_symbol".to_string(), "magenta".to_string());

        Self {
            name: "default".to_string(),
            colors,
        }
    }

    pub fn dark_scheme() -> Self {
        let mut colors = HashMap::new();
        colors.insert("command".to_string(), "blue".to_string());
        colors.insert("argument".to_string(), "white".to_string());
        colors.insert("error".to_string(), "red".to_string());
        colors.insert("success".to_string(), "green".to_string());
        colors.insert("info".to_string(), "cyan".to_string());
        colors.insert("warning".to_string(), "yellow".to_string());
        colors.insert("path".to_string(), "cyan".to_string());
        colors.insert("number".to_string(), "magenta".to_string());
        colors.insert("prompt_user".to_string(), "green".to_string());
        colors.insert("prompt_host".to_string(), "blue".to_string());
        colors.insert("prompt_path".to_string(), "yellow".to_string());
        colors.insert("prompt_symbol".to_string(), "magenta".to_string());

        Self {
            name: "dark".to_string(),
            colors,
        }
    }

    pub fn light_scheme() -> Self {
        let mut colors = HashMap::new();
        colors.insert("command".to_string(), "blue".to_string());
        colors.insert("argument".to_string(), "black".to_string());
        colors.insert("error".to_string(), "red".to_string());
        colors.insert("success".to_string(), "green".to_string());
        colors.insert("info".to_string(), "blue".to_string());
        colors.insert("warning".to_string(), "yellow".to_string());
        colors.insert("path".to_string(), "blue".to_string());
        colors.insert("number".to_string(), "magenta".to_string());
        colors.insert("prompt_user".to_string(), "green".to_string());
        colors.insert("prompt_host".to_string(), "blue".to_string());
        colors.insert("prompt_path".to_string(), "blue".to_string());
        colors.insert("prompt_symbol".to_string(), "black".to_string());

        Self {
            name: "light".to_string(),
            colors,
        }
    }

    pub fn monokai_scheme() -> Self {
        let mut colors = HashMap::new();
        colors.insert("command".to_string(), "cyan".to_string());
        colors.insert("argument".to_string(), "white".to_string());
        colors.insert("error".to_string(), "red".to_string());
        colors.insert("success".to_string(), "green".to_string());
        colors.insert("info".to_string(), "cyan".to_string());
        colors.insert("warning".to_string(), "yellow".to_string());
        colors.insert("path".to_string(), "magenta".to_string());
        colors.insert("number".to_string(), "magenta".to_string());
        colors.insert("prompt_user".to_string(), "green".to_string());
        colors.insert("prompt_host".to_string(), "cyan".to_string());
        colors.insert("prompt_path".to_string(), "yellow".to_string());
        colors.insert("prompt_symbol".to_string(), "red".to_string());

        Self {
            name: "monokai".to_string(),
            colors,
        }
    }

    pub fn get_color(&self, key: &str) -> Option<&String> {
        self.colors.get(key)
    }

    pub fn get_available_schemes() -> Vec<ColorScheme> {
        vec![
            Self::default_scheme(),
            Self::dark_scheme(),
            Self::light_scheme(),
            Self::monokai_scheme(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_scheme() {
        let scheme = ColorScheme::default_scheme();
        assert_eq!(scheme.name, "default");
        assert!(scheme.colors.contains_key("command"));
        assert!(scheme.colors.contains_key("error"));
    }

    #[test]
    fn test_get_color() {
        let scheme = ColorScheme::default_scheme();
        let command_color = scheme.get_color("command");
        assert!(command_color.is_some());
        assert_eq!(command_color.unwrap(), "blue");
    }

    #[test]
    fn test_available_schemes() {
        let schemes = ColorScheme::get_available_schemes();
        assert!(schemes.len() >= 4);
        assert!(schemes.iter().any(|s| s.name == "default"));
        assert!(schemes.iter().any(|s| s.name == "dark"));
        assert!(schemes.iter().any(|s| s.name == "light"));
        assert!(schemes.iter().any(|s| s.name == "monokai"));
    }
}