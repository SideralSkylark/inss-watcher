const INSS_HEADER: &str = "Guia de Pagamento de Contribuição";

pub fn is_inss(text: &str) -> bool {
    text.contains(INSS_HEADER)
}

pub fn extract_reference_date(text: &str) -> Option<(u32, u32)> {
    // Look for the reference date pattern specifically
    // In your text: "20/11/2025 18:1311/2025" 
    // We want to capture "11/2025" which follows a time pattern
    let re = regex::Regex::new(r"\b(\d{1,2})/\d{4}\s+\d{1,2}:\d{2}(\d{1,2})/(\d{4})\b").ok()?;
    
    if let Some(cap) = re.captures(text) {
        // Group 2 is the month, group 3 is the year
        let month: u32 = cap[2].parse().ok()?;
        let year: u32 = cap[3].parse().ok()?;
        
        if (1..=12).contains(&month) {
            return Some((month, year));
        }
    }
    
    // Fallback to the original logic
    let fallback_re = regex::Regex::new(r"([0-1]?\d)/(\d{4})").ok()?;
    
    for cap in fallback_re.captures_iter(text) {
        let month: u32 = cap[1].parse().ok()?;
        let year: u32 = cap[2].parse().ok()?;
        if (1..=12).contains(&month) {
            return Some((month, year));
        }
    }
    
    None
}
