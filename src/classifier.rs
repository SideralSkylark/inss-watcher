use log::info;

pub enum DocumentType {
    Guide,
    Unknown,
}

pub const INSS_HEADER: &str = "Guia de Pagamento de Contribuição";

pub fn is_inss_file(text: &str) -> bool{
    info!("Classifying file");
    let doc_type = classify(text);

    let res = match doc_type {
        DocumentType::Guide => true,
        _ => false,
    };

    return res;
}

fn classify(text: &str) -> DocumentType {
    if text.contains(INSS_HEADER) {
        DocumentType::Guide
    } else {
        DocumentType::Unknown
    }
}

