use chrono::NaiveDate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferencePeriod {
    month: u8,
    year: u16,
}

#[derive(Debug, Clone)]
pub struct InssGuide {
    pub reference_num: String,
    pub reference_period: ReferencePeriod,
    pub amount: Money,
}

#[derive(Debug, Clone)]
pub struct PaymentReceipt {
    pub reference_num: String,
    pub payment_date: NaiveDate,
    pub amount: Money,
}

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
