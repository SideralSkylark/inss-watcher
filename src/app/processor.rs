use std::path::PathBuf;
use log::{debug, info, warn, error};

use crate::domain::classify::DocumentKind;
use crate::domain::guide::{InssGuide, ParsedGuide};
use crate::domain::receipt::{self, ParsedReceipt, PaymentReceipt};
use crate::domain::{classify, guide};
use crate::infra::{fs, ocr, pdf, persistence};

pub fn process_file(path: PathBuf) {
    info!("event=file_processing_started path={:?}", path);

    let mut text = match pdf::extract_text(&path) {
        Ok(t) => t,
        Err(e) => {
            error!("event=processing_failed stage=pdf_extract path={:?} error={}", path, e);
            return;
        }
    };

    if text.trim().is_empty() {
        info!("event=ocr_fallback reason=empty_pdf_text path={:?}", path);

        let img_path = match pdf::pdf_to_img(&path) {
            Ok(p) => p,
            Err(e) => {
                error!("event=processing_failed stage=pdf_render path={:?} error={}", path, e);
            return;
            }
        };

        match ocr::extract_text(&img_path) {
            Ok(ocr_text) => {
                debug!("event=ocr_completed path={:?} text_len={}", img_path, ocr_text.len());
                text = ocr_text;
            },
            Err(e) => {
                error!("event=processing_failed stage=ocr path={:?} error={}", img_path, e);
                return;
            }
        };

        let _ = std::fs::remove_file(&img_path);
    }

    let kind = classify::classify_doc(&text);
    debug!("event=document_classified path={:?} kind={:?} raw_len={}", path, kind, text.len());

    match kind {
        DocumentKind::InssGuide => handle_inss_guide(path, &text),
        DocumentKind::PaymentReceipt => handle_payment_receipt(path, &text),
        DocumentKind::Other => {
            info!("event=document_ignored reason=unsupported_type path={:?}", path);
        }
    }
}

pub fn handle_inss_guide(path: PathBuf, text: &str) {
    info!("event=inss_guide_processing_started path={:?}", path);

    let parsed: ParsedGuide = match guide::parse_guide(&text) {
        Ok(p) => p,
        Err(e) => {
            warn!("event=parsing_failed path={:?} error={}", path, e);
            return;
        },
    };

    if persistence::guide_exists(&parsed) {
        info!("event=resource_already_exists path={:?}", path);
        return;
    }

    let guide: InssGuide = (parsed, path).into();
    persistence::store_guide(&guide);

    try_match_guide(&guide)
}

pub fn handle_payment_receipt(path: PathBuf, text: &str) {
    info!(
        "event=payment_receipt_detected path={:?} raw={}",
        path,
        text
    );

    let parsed: ParsedReceipt = match receipt::parse_receipt(&text) {
        Ok(r) => r,
        Err(e) => {
            warn!("event=parsing_failed path={:?} error={}", path, e);
            return;
        },
    };

    if persistence::receipt_exists(&parsed) {
        info!("event=resource_already_exists path={:?}", path);
        return;
    }

    let receipt: PaymentReceipt = (parsed, path).into();
    persistence::store_receipt(&receipt);

    try_match_receipt(&receipt);
}

fn try_match_guide(guide: &InssGuide) {
    if let Some(receipt) = persistence::find_matching_receipt(guide) {
        info!("event=matching_resource_found path={:?}", receipt.path);
        fs::move_pair(&guide.path, &receipt.path);
    } else {
        let dest = fs::quarentine(&guide.path);
        info!("event=no_matches_found event=guide_moved src={:?} dest={:?}", guide.path, dest);
    }
}

fn try_match_receipt(receipt: &PaymentReceipt) {
    if let Some(guide) = persistence::find_matching_guide(receipt) {
        info!("event=matching_resource_found path={:?}", guide.path);
        fs::move_pair(&receipt.path, &guide.path);
    } else {
        let dest = fs::quarentine(&receipt.path);
        info!("event=no_matches_found event=guide_moved src={:?} dest={:?}", receipt.path, dest);
    }
}
