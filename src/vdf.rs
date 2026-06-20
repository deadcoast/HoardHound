use std::path::PathBuf;

pub fn parse_steam_libraryfolders_vdf(content: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for line in content.lines() {
        let fields = quoted_fields(line);
        if fields.len() >= 2 && fields[0].eq_ignore_ascii_case("path") {
            paths.push(PathBuf::from(unescape_vdf(&fields[1])));
            continue;
        }

        // Older VDF layouts sometimes store numeric keys whose values are paths.
        if fields.len() == 2 && fields[0].chars().all(|c| c.is_ascii_digit()) {
            let maybe_path = unescape_vdf(&fields[1]);
            if looks_like_windows_path(&maybe_path) || maybe_path.contains("SteamLibrary") {
                paths.push(PathBuf::from(maybe_path));
            }
        }
    }

    dedupe(paths)
}

fn quoted_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escaped = false;

    for ch in line.chars() {
        if !in_quote {
            if ch == '"' {
                in_quote = true;
                current.clear();
            }
            continue;
        }

        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => {
                in_quote = false;
                fields.push(current.clone());
            }
            _ => current.push(ch),
        }
    }

    fields
}

fn unescape_vdf(value: &str) -> String {
    value
        .replace("\\\\", "\\")
        .replace("\\/", "/")
        .replace("\\\"", "\"")
}

fn looks_like_windows_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3 && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/')
}

fn dedupe(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for path in paths {
        let key = path.to_string_lossy().to_ascii_lowercase();
        if seen.insert(key) {
            out.push(path);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modern_libraryfolders() {
        let content = r#"
        "libraryfolders"
        {
            "0"
            {
                "path" "C:\\Program Files (x86)\\Steam"
            }
            "1"
            {
                "path" "D:\\SteamLibrary"
            }
        }
        "#;
        let paths = parse_steam_libraryfolders_vdf(content);
        assert_eq!(paths.len(), 2);
        assert!(paths[1].to_string_lossy().contains("D:\\SteamLibrary"));
    }
}
