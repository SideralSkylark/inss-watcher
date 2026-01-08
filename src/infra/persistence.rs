use std::str::FromStr;

use chrono::NaiveDate;

use crate::domain::guide::{ParsedGuide, ReferencePeriod};
use crate::domain::money::Money;
use crate::domain::receipt::ParsedReceipt;

pub fn guide_exists(guide: &ParsedGuide) -> bool {
    true
}

pub fn store_guide(guide: &ParsedGuide) -> Option<bool> {
    Some(true)
}

pub fn find_matching_receipt(guide: &ParsedGuide) -> Option<ParsedReceipt> {
    let date = NaiveDate::from_ymd_opt(2025, 12, 26).unwrap();
    Some(ParsedReceipt { reference_num: String::from("10"), payment_date: date, amount: Money { cents: 12 } })
}

pub fn receipt_exists(receipt: &ParsedReceipt) -> bool {
    true
}

pub fn store_receipt(receipt: &ParsedReceipt) -> Option<bool> {
    Some(true)
}

pub fn find_matching_guide(receipt: &ParsedReceipt) -> Option<ParsedGuide> {
    Some(ParsedGuide { reference_num: String::from(""), contributor_num: String::from(""), reference_period: ReferencePeriod {month: 01, year: 2026}, amount: Money { cents: 100 } })
}
