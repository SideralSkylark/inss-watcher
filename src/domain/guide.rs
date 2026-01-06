use anyhow::Context;
use crate::domain::money::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferencePeriod {
    pub month: u8,
    pub year: u16,
}

#[derive(Debug, Clone)]
pub struct InssGuide {
    pub reference_num: String,
    pub contributor_num: String,
    pub reference_period: ReferencePeriod,
    pub amount: Money,
}

pub fn parse_guide(text: &str) -> anyhow::Result<InssGuide> {
    Ok(
        InssGuide { 
            reference_num: extract_guide_reference(text).context("missing guide reference")?, 
            contributor_num: extract_contributor_num(text).context("missing contributor number")?,
            reference_period: extract_reference_period(text).context("missing reference period")?, 
            amount: extract_amount(text).context("missing payment amount")?, 
        }
    )
}

fn extract_guide_reference(text: &str) -> Option<String> {
    let re = regex::Regex::new(
        r"(?is)Número\s+da\s+Guia[^\d]{0,30}(\d{8,12})"
    ).ok()?;

    re.captures_iter(text)
        .last()
        .map(|cap| cap[1].to_string())
}

pub fn extract_contributor_num(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?is)Guia de Pagamento de Contribuição\s*-\s*GPC.*?(\d{8,12}).*?Número do Contribuinte").ok()?;

    let caps = re.captures(text)?;
    Some(caps[1].to_string())
}

fn extract_reference_period(text: &str) -> Option<ReferencePeriod> {
    let re = regex::Regex::new(r"(0?[1-9]|1[0-2])/(\d{4})").ok()?;

    re.captures_iter(text)
        .last()
        .and_then(|cap| {
            Some(ReferencePeriod {
                month: cap[1].parse().ok()?,
                year: cap[2].parse().ok()?,
            })
        })
}

fn extract_amount(text: &str) -> Option<Money> {
    let re = regex::Regex::new(r"([\d.,]+)\s*MT").ok()?;

    let raw = re.captures(text)?.get(1)?.as_str();
    Money::from_str(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_reference() {
        let text = "Data limite de PagamentoNúmero da Guia
                    123456789";
        assert_eq!(
            extract_guide_reference(text),
            Some("123456789".to_string())
        )
    }

    #[test]
    fn extracts_reference_period_simple() {
        let text = "Referência 10/2025";

        let period = extract_reference_period(text).unwrap();

        assert_eq!(period.month, 10);
        assert_eq!(period.year, 2025);
    }

    #[test]
    fn takes_last_reference_period_when_multiple_exist() {
        let text = "01/2024 texto intermédio 10/2025";

        let period = extract_reference_period(text).unwrap();

        assert_eq!(period.month, 10);
        assert_eq!(period.year, 2025);
    }

    #[test]
    fn reference_period_returns_none_when_absent() {
        let text = "Documento sem mês e ano";

        assert!(extract_reference_period(text).is_none());
    }

    #[test]
    fn extracts_amount_with_comma_decimal() {
        let text = "Valor Total da Guia 721,70 MT";

        let amount = extract_amount(text).unwrap();

        assert_eq!(amount.cents, 72170);
    }

    #[test]
    fn extracts_amount_with_dot_thousands() {
        let text = "Valor Total da Guia 1.234,56 MT";

        let amount = extract_amount(text).unwrap();

        assert_eq!(amount.cents, 123456);
    }

    #[test]
    fn amount_returns_none_when_missing() {
        let text = "Sem valores monetários";

        assert!(extract_amount(text).is_none());
    }

    #[test]
    fn extracts_contributor_number() {
        let text = "
            Guia de Pagamento de Contribuição - GPC
            915732100
            Número do Contribuinte
        ";

        assert_eq!(
            extract_contributor_num(text),
            Some("915732100".to_string())
        );
    }

    #[test]
    fn parses_complete_guide_successfully() {
        let text = r#"
            Guia de Pagamento de Contribuição - GPC
            915732100
            Número do Contribuinte
            721,70 MT
            Valor Total da Guia
            Data limite de PagamentoNúmero da Guia
            115320342
            03/11/2025 14:4610/2025
        "#;

        let guide = parse_guide(text).unwrap();

        assert_eq!(guide.reference_num, "115320342");
        assert_eq!(guide.reference_period.month, 10);
        assert_eq!(guide.reference_period.year, 2025);
        assert_eq!(guide.amount.cents, 72170);
    }

    #[test]
    fn parse_guide_fails_when_reference_missing() {
        let text = "721,70 MT 10/2025";

        let err = parse_guide(text).unwrap_err();

        assert!(err.to_string().contains("missing guide reference"));
    }

    #[test]
    fn parse_guide_fails_when_contributor_missing() {
        let text = "
            Guia de Pagamento de Contribuição - GPC
            Número do Contribuinte
            ";

        let err = parse_guide(text).unwrap_err();

        assert!(err.to_string().contains("missing contributor number"));
    }

    #[test]
    fn parse_guide_fails_when_reference_period_missing() {
        let text = r#"
            Data limite de PagamentoNúmero da Guia
            115320342
            721,70 MT
        "#;

        let err = parse_guide(text).unwrap_err();

        assert!(err.to_string().contains("missing reference period"));
    }

    #[test]
    fn parse_guide_fails_when_amount_missing() {
        let text = r#"
            Data limite de PagamentoNúmero da Guia
            115320342
            10/2025
        "#;

        let err = parse_guide(text).unwrap_err();

        assert!(err.to_string().contains("missing payment amount"));
    }

} 
