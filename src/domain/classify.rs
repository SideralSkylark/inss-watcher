#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    InssGuide,
    PaymentReceipt,
    Other,
}

pub fn classify_doc(text: &str) -> DocumentKind {
    let lower = text.to_lowercase(); 
    
    if lower.contains("guia de pagamento de contribuição") {
        DocumentKind::InssGuide
    } else if lower.contains("comprovativo") && lower.contains("pagamento") {
        DocumentKind::PaymentReceipt
    } else {
        DocumentKind::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_classify_guide() {
        let text = "Guia de Pagamento de Contribuição";
        assert!(matches!(
            classify_doc(text),
            DocumentKind::InssGuide
        ));

    }

    #[test]
    fn should_classify_receipt() {
        let text = "comprovativo do pagamento da guia";
        assert!(matches!(
            classify_doc(text),
            DocumentKind::PaymentReceipt
        ))
    }
}
