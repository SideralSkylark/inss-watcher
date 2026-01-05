#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    InssGuide,
    PaymentReceipt,
    Other,
}

pub fn classify_doc(text: &str) -> DocumentKind {
    let lower = text.to_lowercase(); 
    
    if lower.contains("Guia de Pagamento de Contribuição") {
        DocumentKind::InssGuide
    } else if lower.contains("comprovativo") && lower.contains("pagamento") {
        DocumentKind::PaymentReceipt
    } else {
        DocumentKind::Other
    }
}

