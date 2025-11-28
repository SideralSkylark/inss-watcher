use regex::Regex;

pub fn parse_reference_date(text: &str) -> Option<(u32, u32)> {
    let re = Regex::new(r"Refer[eê]ncia[:\s]*([0-1]?\d)/(\d{4})").ok()?;
    let caps = re.captures(text)?;

    let month: u32 = caps.get(1)?.as_str().parse().ok()?;
    let year: u32 = caps.get(2)?.as_str().parse().ok()?;

    if month >= 1 && month <= 12 {
        Some((month, year))
    } else {
        None
    }
}
