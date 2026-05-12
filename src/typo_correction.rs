use crate::config::DOMAIN_CORRECTIONS;
use crate::types::EmailCellResult;
use regex::Regex;

pub fn correct_domain_typo(domain: &str) -> (String, bool, Option<String>) {
    let original = domain.to_lowercase().trim().to_string();
    let mut corrected = original.clone();
    let mut correction_desc = None;
    
    for (pattern, replacement) in DOMAIN_CORRECTIONS.iter() {
        if pattern.is_match(&corrected) {
            let new_corrected = pattern.replace_all(&corrected, *replacement).to_string();
            if new_corrected != corrected {
                correction_desc = Some(format!("{} -> {}", corrected, new_corrected));
                corrected = new_corrected;
                break;
            }
        }
    }
    
    if corrected != original {
        corrected = corrected.trim_end_matches('.').to_string();
        if corrected.matches('@').count() > 1 {
            let parts: Vec<&str> = corrected.split('@').collect();
            if parts.len() >= 2 {
                corrected = format!("{}@{}", parts[0], parts[parts.len() - 1]);
            }
        }
    }
    
    let was_corrected = corrected != original;
    (corrected, was_corrected, correction_desc)
}

pub fn normalize_email_with_correction(email: &str) -> Option<EmailCellResult> {
    if email.is_empty() {
        return None;
    }
    
    let email = email.trim().to_lowercase();
    
    // Handle comma/semicolon separated emails
    if email.contains(',') || email.contains(';') {
        let emails: Vec<&str> = email
            .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .collect();
        if let Some(first) = emails.first() {
            return normalize_email_with_correction(first);
        }
        return None;
    }
    
    if !email.contains('@') {
        return None;
    }
    
    let parts: Vec<&str> = email.rsplitn(2, '@').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let (domain, local_part) = (parts[0], parts[1]);
    
    // Validate local part
    if local_part.is_empty() 
        || local_part.len() > 64 
        || !crate::config::LOCAL_PART_RE.is_match(local_part) 
    {
        return None;
    }
    
    // Correct domain typos
    let (corrected_domain, was_corrected, correction_desc) = correct_domain_typo(domain);
    let corrected_email = format!("{}@{}", local_part, corrected_domain);
    
    // Final validation
    if !crate::config::EMAIL_RE.is_match(&corrected_email) || corrected_email.len() > 254 {
        return None;
    }
    
    Some((corrected_email, was_corrected, correction_desc))
}