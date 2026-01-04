use crate::domain::{classify::classify, record::{InssRecord, Money, ReferencePeriod}};

pub fn extract_record(text: &str) -> InssRecord {
    let kind = classify(text);

    InssRecord {
        kind,
        reference: extract_reference(text),
        contributor: extract_contributor(text, kind),
        amount: extract_amout(text),
        raw_len: text.len(),
    }
}

fn extract_reference(text: &str) -> ReferencePeriod {
    let re = Regex::new(r"([0-1]?\d)/(\d{4})").ok()?;

    re.captures_iter(text).find_map(|cap| {
        let month: u32 = cap[1].parse().ok()?;
        let year: u32 = cap[2].parse().ok()?;

        (1..=12).contains(&month).then_some(ReferencePeriod { month, year })
    })
}

fn extract_contributor(text: &str, kind: &DocumentKind) -> Option<String> {
    match kind {
        DocumentKind::InssGuide => extract_contributor
    }
}

fn extract_amout(text: &str) -> Option<Money> {

}
