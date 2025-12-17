const INSS_HEADER: &str = "Guia de Pagamento de Contribuição";

pub fn is_inss(text: &str) -> bool {
    text.contains(INSS_HEADER)
}

pub fn extract_reference_date(text: &str) -> Option<(u32, u32)> {
    let re = regex::Regex::new(r"([0-1]?\d)/(\d{4})").ok()?;

    for cap in re.captures_iter(text) {
        let month: u32 = cap[1].parse().ok()?;
        let year: u32 = cap[2].parse().ok()?;
        if (1..=12).contains(&month) {
            return Some((month, year));
        }
    }
    None
}
