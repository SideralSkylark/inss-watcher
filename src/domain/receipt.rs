use anyhow::Context;
use chrono::NaiveDate;
use crate::domain::money::Money;

#[derive(Debug, Clone)]
pub struct PaymentReceipt {
    pub reference_num: String,
    pub payment_date: NaiveDate,
    pub amount: Money,
}

pub fn parse_receipt(text: &str) -> anyhow::Result<PaymentReceipt> {
    Ok(PaymentReceipt { 
        reference_num: extract_reference_num(text).context("missing reference number")?, 
        payment_date: extract_payment_date(text).context("missing payment date")?,
        amount: extract_amount(text).context("missing payment amount")?,
    })
}

fn extract_reference_num(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"Referência\s+(\d{9})").ok()?;
    Some(re.captures(text)?.get(1)?.as_str().to_string())
}

fn extract_payment_date(text: &str) -> Option<NaiveDate> {
    let re = regex::Regex::new(r"(\d{2}/\d{2}/\d{4})").ok()?;
    let raw = re.captures(text)?.get(1)?.as_str();
    NaiveDate::parse_from_str(raw, "%d/%m/%Y").ok()
}

fn extract_amount(text: &str) -> Option<Money> {
    let re = regex::Regex::new(r"Montante pagamento\s+([\d.,]+)").ok()?;
    let raw = re.captures(text)?.get(1)?.as_str();
    Money::from_str(raw)
}
