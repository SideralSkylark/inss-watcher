use crate::domain::record::DocumentKind;

pub fn classify(text: &str) -> DocumentKind {
    let lower = text.to_lowercase();

    if lower.contains("guia de pagamento de contribuição") {
        DocumentKind::InssGuide
    } else if looks_like_payment(&lower) {
        DocumentKind::PaymentReceipt
    } else {
        DocumentKind::Other
    }
}

pub fn looks_like_payment(text: &str) -> bool {
    let indicators = [
        "comprovativo",
        "pagamento",
        "netplus",
        "montante",
        "pagamento ao estado",
        "inss",
    ];

    indicators.iter().any(|k| text.contains(k))
}
