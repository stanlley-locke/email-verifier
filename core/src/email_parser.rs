use crate::typo_correction::normalize_email_with_correction;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref COLUMN_PATTERN_RE: Regex = Regex::new(r"(?i).*email.*").unwrap();
}

pub fn extract_emails_from_cell(cell_value: &str) -> Vec<(String, bool, Option<String>)> {
    if cell_value.is_empty() {
        return Vec::new();
    }
    
    let raw_emails: Vec<&str> = cell_value
        .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .collect();
    
    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();
    
    for email in raw_emails {
        if let Some((normalized, corrected, desc)) = normalize_email_with_correction(email) {
            if !seen.contains(&normalized) {
                seen.insert(normalized.clone());
                results.push((normalized, corrected, desc));
            }
        }
    }
    
    results
}

pub fn identify_email_columns(headers: &[String], pattern: &str) -> anyhow::Result<Vec<(usize, String)>> {
    let pattern_re = Regex::new(pattern)
        .map_err(|e| anyhow::anyhow!("Invalid column pattern regex: {}", e))?;
    
    Ok(headers
        .iter()
        .enumerate()
        .filter_map(|(idx, header)| {
            if !header.is_empty() && pattern_re.is_match(header) {
                Some((idx + 1, header.clone()))
            } else {
                None
            }
        })
        .collect())
}

pub fn count_non_empty_email_cells(
    rows: &[Vec<String>],
    email_columns: &[(usize, String)],
) -> usize {
    let mut count = 0;
    for row in rows.iter().skip(1) {
        for (col_idx, _) in email_columns {
            if let Some(cell) = row.get(*col_idx - 1) {
                if !cell.trim().is_empty() {
                    count += 1;
                }
            }
        }
    }
    count
}