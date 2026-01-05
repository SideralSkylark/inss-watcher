use chrono::NaiveDate;

use crate::domain::{guide::{InssGuide, ReferencePeriod}, receipt::PaymentReceipt};

pub fn match_docs(guide: &InssGuide, receipt: &PaymentReceipt) -> bool {
    if guide.reference_num == receipt.reference_num
        && guide.amount.cents == receipt.amount.cents
        && within_period(guide.reference_period, receipt.payment_date) {
        return true;
    }
    false
}

fn within_period(reference_period: ReferencePeriod, payment_date: NaiveDate) -> bool {
   return true;
}



