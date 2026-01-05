use chrono::NaiveDate;

use crate::domain::{guide::{InssGuide, ReferencePeriod}, receipt::PaymentReceipt};

pub fn match_docs(guide: &InssGuide, receipt: &PaymentReceipt) -> bool {
    guide.reference_num == receipt.reference_num
    && guide.amount.cents == receipt.amount.cents
    && within_period(guide.reference_period, receipt.payment_date)
}

fn within_period(reference_period: ReferencePeriod, payment_date: NaiveDate) -> bool {
    let deadline = payment_deadline(reference_period);

    payment_date <= deadline
}

fn payment_deadline(reference_period: ReferencePeriod) -> NaiveDate {
    let ReferencePeriod { month, year } = reference_period;

    let first_day = NaiveDate::from_ymd_opt(year as i32, month as u32, 1).expect("valid reference period");

    let (next_month, next_year) = if month == 12 {
        (1, year + 1)
    } else {
        (month + 1, year)
    };

    NaiveDate::from_ymd_opt(next_year as i32, next_month as u32, 10).expect("valid deadline date")
}
