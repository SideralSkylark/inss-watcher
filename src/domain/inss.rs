const INSS_HEADER: &str = "Guia de Pagamento de Contribuição";

pub fn is_inss(text: &str) -> bool {
    text.contains(INSS_HEADER)
}

pub fn extract_reference_date(text: &str) -> Option<(u32, u32)> {
    let re = regex::Regex::new(r"\b(\d{1,2})/\d{4}\s+\d{1,2}:\d{2}(\d{1,2})/(\d{4})\b").ok()?;
    
    if let Some(cap) = re.captures(text) {
        let month: u32 = cap[2].parse().ok()?;
        let year: u32 = cap[3].parse().ok()?;
        
        if (1..=12).contains(&month) {
            return Some((month, year));
        }
    }
    
    let fallback_re = regex::Regex::new(r"([0-1]?\d)/(\d{4})").ok()?;
    
    for cap in fallback_re.captures_iter(text) {
        let month: u32 = cap[1].parse().ok()?;
        let year: u32 = cap[2].parse().ok()?;
        if (1..=12).contains(&month) {
            return Some((month, year));
        }
    }
    
    None
}

// contr num is 9 digits long, its between Número do Contribuinte and Guia de Pagamento de Contribuição - GPC
pub fn extract_contributor_num(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?is)Guia de Pagamento de Contribuição\s*-\s*GPC.*?(\d{9}).*?Número do Contribuinte").ok()?;

    let caps = re.captures(text)?;
    Some(caps[1].to_string())
}

// guide num is 9 digits long, its between Data limite de PagamentoNúmero da Guia and Autenticação Bancária
pub fn extract_guide_num(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?is)Data limite de PagamentoNúmero da Guia.*?(\d{9}).*?Autenticação Bancária").ok()?;

    let caps = re.captures(text)?;
    Some(caps[1].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_inss_header() {
        let text = "Guia de Pagamento de Contribuição";
        assert!(is_inss(text));
    }

    #[test]
    fn reject_non_inss() {
        let text = "Random doc";
        assert!(!is_inss(text));
    }

    #[test]
    fn extracts_reference_date_primary() {
        let text = "20/11/2025 18:1311/2025";
        assert_eq!(extract_reference_date(text), Some((11, 2025)));
    }

    #[test]
    fn extracts_reference_date_fallback() {
        let text = "Referência 03/2024";
        assert_eq!(extract_reference_date(text), Some((3, 2024)));
    }

    #[test]
    fn extracts_guide_number() {
        let text = "
            Data limite de PagamentoNúmero da Guia
            123456789
            Autenticação Bancária
        ";

        assert_eq!(
            extract_guide_num(text), 
            Some("123456789".to_string())
        )
    }

    #[test]
    fn extracts_contributor_num() {
        let text = "
            Guia de Pagamento de Contribuição - GPC
            123456789
            Número do Contribuinte
        ";

        assert_eq!(
            extract_contributor_num(text),
            Some("123456789".to_string())
        )
    }
}

