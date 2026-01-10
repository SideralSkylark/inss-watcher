use std::path::PathBuf;

use anyhow::Context;
use chrono::NaiveDate;
use crate::domain::money::Money;

#[derive(Debug, Clone)]
pub struct ParsedReceipt {
    pub reference_num: String,
    pub payment_date: NaiveDate,
    pub amount: Money,
}

#[derive(Debug, Clone)]
pub struct PaymentReceipt {
    pub reference_num: String,
    pub payment_date: NaiveDate,
    pub amount: Money,
    pub path: PathBuf,
}

impl From<(ParsedReceipt, PathBuf)> for PaymentReceipt {
    fn from((parsed, path): (ParsedReceipt, PathBuf)) -> Self {
        Self {
            reference_num: parsed.reference_num,
            payment_date: parsed.payment_date,
            amount: parsed.amount,
            path,
        }
    }
}

pub fn parse_receipt(text: &str) -> anyhow::Result<ParsedReceipt> {
    Ok(ParsedReceipt { 
        reference_num: extract_reference_num(text).context("missing reference number")?, 
        payment_date: extract_payment_date(text).context("missing payment date")?,
        amount: extract_amount(text).context("missing payment amount")?,
    })
}

fn extract_reference_num(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"Referência\s+(\d{8,12})").ok()?;
    Some(re.captures(text)?.get(1)?.as_str().to_string())
}

fn extract_payment_date(text: &str) -> Option<NaiveDate> {
    let re = regex::Regex::new(r"(\d{2}/\d{2}/\d{4})").ok()?;
    let raw = re.captures(text)?.get(1)?.as_str();
    NaiveDate::parse_from_str(raw, "%d/%m/%Y").ok()
}

fn extract_amount(text: &str) -> Option<Money> {
    let re = regex::Regex::new(
        r"Montante pagamento\s*[-–—:]?\s*([\d.,]+)"
    ).ok()?;

    let raw = re.captures(text)?.get(1)?.as_str();
    Money::from_str(raw)
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn extracts_reference_number_9() {
        let text = "
            Referência 117450766
            Montante pagamento 698,75
        ";

        assert_eq!(
            extract_reference_num(text),
            Some("117450766".to_string())
        );
    }

    #[test]
    fn extracts_reference_number_8() {
        let text = "
            Referência 11745076
            Montante pagamento 698,75
        ";

        assert_eq!(
            extract_reference_num(text),
            Some("11745076".to_string())
        );
    }

    #[test]
    fn reference_number_returns_none_when_missing() {
        let text = "Documento sem referência";

        assert!(extract_reference_num(text).is_none());
    }

    #[test]
    fn extracts_payment_date() {
        let text = "
            Data processamento 26/12/2025
            Montante pagamento 698,75
        ";

        let date = extract_payment_date(text).unwrap();

        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2025, 12, 26).unwrap()
        );
    }

    #[test]
    fn extracts_first_date_when_multiple_dates_exist() {
        let text = "
            Data processamento 25/12/2025
            Data valor 26/12/2025
        ";

        let date = extract_payment_date(text).unwrap();

        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2025, 12, 25).unwrap()
        );
    }

    #[test]
    fn payment_date_returns_none_when_missing() {
        let text = "Sem datas relevantes";

        assert!(extract_payment_date(text).is_none());
    }

    #[test]
    fn extracts_payment_amount() {
        let text = "
            Montante pagamento 698,75
            Moeda pagamento MZN
        ";

        let amount = extract_amount(text).unwrap();

        assert_eq!(amount.cents, 69875);
    }

    #[test]
    fn extracts_payment_amount_with_em_dash() {
        let text = "
            Montante pagamento — 698,75
        ";

        let amount = extract_amount(text).unwrap();
        assert_eq!(amount.cents, 69875);
    }


    #[test]
    fn extracts_payment_amount_with_thousands_separator() {
        let text = "
            Montante pagamento 1.234,56
        ";

        let amount = extract_amount(text).unwrap();

        assert_eq!(amount.cents, 123456);
    }

    #[test]
    fn payment_amount_returns_none_when_missing() {
        let text = "Sem montantes monetários";

        assert!(extract_amount(text).is_none());
    }

    #[test]
    fn parses_complete_receipt_successfully() {
        let text = r#"
            Comprovativo de operação
            Pagamento ao estado INSS

            Referência 117450766
            Montante pagamento 698,75

            Data processamento 26/12/2025
        "#;

        let receipt = parse_receipt(text).unwrap();

        assert_eq!(receipt.reference_num, "117450766");
        assert_eq!(
            receipt.payment_date,
            NaiveDate::from_ymd_opt(2025, 12, 26).unwrap()
        );
        assert_eq!(receipt.amount.cents, 69875);
    }

    #[test]
    fn parse_receipt_fails_when_reference_missing() {
        let text = "
            Montante pagamento 698,75
            Data processamento 26/12/2025
        ";

        let err = parse_receipt(text).unwrap_err();

        assert!(err.to_string().contains("missing reference number"));
    }

    #[test]
    fn parse_receipt_fails_when_payment_date_missing() {
        let text = "
            Referência 117450766
            Montante pagamento 698,75
        ";

        let err = parse_receipt(text).unwrap_err();

        assert!(err.to_string().contains("missing payment date"));
    }

    #[test]
    fn parse_receipt_fails_when_amount_missing() {
        let text = "
            Referência 117450766
            Data processamento 26/12/2025
        ";

        let err = parse_receipt(text).unwrap_err();

        assert!(err.to_string().contains("missing payment amount"));
    }
}

