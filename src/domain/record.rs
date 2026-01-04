#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    InssGuide,
    PaymentReceipt,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencePeriod {
    pub month: u32,
    pub year: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Currency {
    MZN,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    pub cents: i64,
    pub currency: Currency,
}

#[derive(Debug, Clone)]
pub struct InssRecord {
    pub kind: DocumentKind,

    pub reference: Option<ReferencePeriod>,
    pub contributor: Option<String>,
    pub amount: Option<Money>,

    pub raw_len: usize,
}
