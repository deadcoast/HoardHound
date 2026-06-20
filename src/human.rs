use anyhow::{bail, Result};

const KIB: f64 = 1024.0;
const MIB: f64 = KIB * 1024.0;
const GIB: f64 = MIB * 1024.0;
const TIB: f64 = GIB * 1024.0;

pub fn format_size(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= TIB {
        format!("{:.1} TB", b / TIB)
    } else if b >= GIB {
        format!("{:.1} GB", b / GIB)
    } else if b >= MIB {
        format!("{:.1} MB", b / MIB)
    } else if b >= KIB {
        format!("{:.1} KB", b / KIB)
    } else {
        format!("{} B", bytes)
    }
}

pub fn parse_size(input: &str) -> Result<u64> {
    let s = input
        .trim()
        .to_ascii_lowercase()
        .replace('_', "")
        .replace(' ', "");
    if s.is_empty() {
        bail!("size cannot be empty");
    }

    let split_at = s
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(s.len());
    let (number, unit) = s.split_at(split_at);
    if number.is_empty() {
        bail!("invalid size '{}': expected a number", input);
    }

    let value: f64 = number.parse()?;
    if !value.is_finite() || value < 0.0 {
        bail!(
            "invalid size '{}': expected a positive finite number",
            input
        );
    }

    let multiplier = match unit {
        "" | "b" | "byte" | "bytes" => 1.0,
        "k" | "kb" | "kib" => KIB,
        "m" | "mb" | "mib" => MIB,
        "g" | "gb" | "gib" => GIB,
        "t" | "tb" | "tib" => TIB,
        other => bail!("invalid size unit '{}'; try MB, GB, or TB", other),
    };

    Ok((value * multiplier) as u64)
}

pub fn truncate_middle(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars || max_chars < 8 {
        return s.to_string();
    }
    let left = (max_chars - 1) / 2;
    let right = max_chars - 1 - left;
    let prefix: String = s.chars().take(left).collect();
    let suffix: String = s.chars().skip(count - right).collect();
    format!("{}…{}", prefix, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_sizes() {
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1.5 GB").unwrap(), 1_610_612_736);
        assert_eq!(parse_size("2tb").unwrap(), 2_199_023_255_552);
    }
}
