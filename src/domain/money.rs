#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    pub cents: i64,
}

impl Money {
    pub fn from_str(raw: &str) -> Option<Self> {
        let normalized = raw
            .replace('.', "")
            .replace(',', ".");

        let value: f64 = normalized.parse().ok()?;

        Some(Self {
            cents: (value * 100.0) as i64,
        })
    }
}
