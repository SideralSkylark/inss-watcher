use anyhow::Context;

use crate::domain::money::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferencePeriod {
    month: u8,
    year: u16,
}

#[derive(Debug, Clone)]
pub struct InssGuide {
    pub reference_num: String,
    pub reference_period: ReferencePeriod,
    pub amount: Money,
}

pub fn parse_guide(text: &str) -> anyhow::Result<InssGuide> {
    Ok(
        InssGuide { 
            reference_num: extract_guide_reference(text).context("missing guide reference"), 
            reference_period: extract_reference_period(text).context("missing reference period"), 
            amount: extract_amount(text).context("missing payment amount"), 
        }
    )
}

// reference is a 9 digit long unsigned int that apears after Data limite de PagamentoNúmero da Guia
fn extract_guide_reference(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"Data limite de PagamentoNúmero da Guia\s*(\d{9})").ok()?;

    let caps = re.captures(text)?;
    Some(caps[1].to_string())

}

// contr num is 9 digits long, its between Número do Contribuinte and Guia de Pagamento de Contribuição - GPC
pub fn extract_contributor_num(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?is)Guia de Pagamento de Contribuição\s*-\s*GPC.*?(\d{9}).*?Número do Contribuinte").ok()?;

    let caps = re.captures(text)?;
    Some(caps[1].to_string())
}

fn extract_reference_period(text: &str) -> Option<ReferencePeriod> {
    let re = regex::Regex::new(r"\b(0?[1-9]|1[0-2])/(\d{4})\b").ok()?;

    re.captures_iter(text)
        .last()
        .map(|cap| ReferencePeriod { 
            month: cap[1].parse().ok()?, 
            year: cap[2].parse().ok()?, 
        })
}

fn extract_amount(text: &str) -> Option<Money> {
    let re = regex::Regex::new(r"([\d.,]+)\s*MT").ok()?;

    let raw = re.captures(text)?.get(1)?.as_str();
    Money::from_str(raw)
}

