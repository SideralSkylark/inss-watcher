use regex::Regex;
use log::info;

pub fn parse_reference_date(text: &str) -> Option<(u32, u32)> {
    info!("Parsing reference date");
    let re = Regex::new(r"([0-1]?\d)/(\d{4})").ok()?;
    
    for caps in re.captures_iter(text) {
        let month: u32 = caps.get(1)?.as_str().parse().ok()?;
        let year: u32 = caps.get(2)?.as_str().parse().ok()?;

        if (1..=12).contains(&month) {
            info!("Found reference date");
            return Some((month, year));
        }
    }

    info!("No reference date found");
    None
}
