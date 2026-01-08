use std::str::FromStr;

use chrono::NaiveDate;

use crate::domain::guide::{InssGuide, ParsedGuide, ReferencePeriod};
use crate::domain::money::Money;
use crate::domain::receipt::{ParsedReceipt, PaymentReceipt};

pub fn guide_exists(guide: &ParsedGuide) -> bool {
    true
}

pub fn store_guide(guide: &InssGuide) -> Option<bool> {
    Some(true)
}

pub fn find_matching_receipt(guide: &InssGuide) -> Option<PaymentReceipt> {
    let date = NaiveDate::from_ymd_opt(2025, 12, 26).unwrap();
    Some(PaymentReceipt { reference_num: String::from("10"), payment_date: date, amount: Money { cents: 12 }, path: guide.path.clone() })
}

pub fn receipt_exists(receipt: &ParsedReceipt) -> bool {
    true
}

pub fn store_receipt(receipt: &PaymentReceipt) -> Option<bool> {
    Some(true)
}

pub fn find_matching_guide(receipt: &PaymentReceipt) -> Option<InssGuide> {
    Some(InssGuide { reference_num: String::from(""), contributor_num: String::from(""), reference_period: ReferencePeriod {month: 01, year: 2026}, amount: Money { cents: 100 }, path: receipt.path.clone() })
}
