use crate::domain::guide::InssGuide;
use crate::domain::receipt::PaymentReceipt;

pub fn guide_exists(guide: &InssGuide) -> bool {
    true
}

pub fn store_guide(guide: &InssGuide) -> Option<bool> {
    Some(true)
}

pub fn receipt_exists(receipt: &PaymentReceipt) -> bool {
    true
}

pub fn store_receipt(receipt: &PaymentReceipt) -> Option<bool> {
    Some(true)
}
