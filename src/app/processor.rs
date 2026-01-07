use std::path::PathBuf;
use log::{debug, info, warn, error};

use crate::domain::classify::DocumentKind;
use crate::domain::guide::InssGuide;
use crate::domain::receipt::PaymentReceipt;
use crate::domain::{classify, guide, matcher};
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

    let guide: InssGuide = match guide::parse_guide(&text) {
        Ok(g) => g,
        Err(e) => {
            warn!("event=parsing_failed path={:?} error={}", path, e);
            return;
        },
    };

    if persistence::guide_exists(guide) {
        info!("event=resource_already_exists path={:?}", path);
        return;
    }

    persistence::store_guide(guide);

    try_match_guide(guide, path)
}

pub fn handle_payment_receipt(path: PathBuf, text: &str) {
    info!(
        "event=payment_receipt_detected path={:?} raw={}",
        path,
        text
    );
}

fn try_match_guide(guide: InssGuide, path: PathBuf) {
        let matching_resource: PaymentReceipt = match persistence::query_matching() {
        Some(receipt) => {
            info("event=matching_resource_found path={:?}", matching_resource.path);
            fs::move_pair(path, matching_resource.path);
        } None => {
            let quarentine_dest = fs::quarentine(path);
            info!("event=no_matches_found event=guide_moved src={:?} dest={:?}", path, quarentine_dest);
        }
    }

}

fn try_match_receipt(receipt: PaymentReceipt, path: PathBuf) {

}
