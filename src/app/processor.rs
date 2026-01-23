use std::path::PathBuf;
use log::{debug, info, warn, error};

use crate::domain::classify::DocumentKind;
use crate::domain::guide::{InssGuide, ParsedGuide};
use crate::domain::receipt::{self, ParsedReceipt, PaymentReceipt};
use crate::domain::{classify, guide};
use crate::infra::fs::MovedPairPaths;
use crate::infra::persistence::StoreOutcome;
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
    
    let outcome = match persistence::store_guide(&guide) {
        Ok(o) => o,
        Err(e) => {
            error!("event=storing_failed kind=guide path={:?} error={}", guide.path, e);
            return;
        },
    };

    if matches!(outcome, StoreOutcome::Inserted) {
        info!("event=guide_stored path={:?}", guide.path);
        try_match_guide(&guide);
    }
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

    let outcome = match persistence::store_receipt(&receipt) {
        Ok(o) => o,
        Err(e) => {
            error!("event=storing_failed kind=guide error={} path={:?}", e, receipt.path);
            return;
        },
    };

    if matches!(outcome, StoreOutcome::Inserted) {
        info!("event=receipt_stored path={:?}", receipt.path);
        try_match_receipt(&receipt);
    }
}

fn try_match_guide(guide: &InssGuide) {
    if let Some(receipt) = persistence::find_matching_receipt(guide) {
        info!("event=matching_resource_found path={:?}", receipt.path);

        match fs::move_pair(&guide, &receipt) {
            Ok(moved) => {
                persist_moved_pair(&guide, &receipt, moved);
            } 
            Err(e) => {
                error!("event=fs_pair_move_failed error={}", e);
            }
        }
    } else {
        debug!(
            "event=no_matching_yet kind=guide ref={}",
            guide.reference_num
        );
    }
}

fn try_match_receipt(receipt: &PaymentReceipt) {
    if let Some(guide) = persistence::find_matching_guide(receipt) {

        info!("event=matching_resource_found path={:?}", guide.path);
        match fs::move_pair(&guide, &receipt) {
            Ok(moved) => {
                persist_moved_pair(&guide, &receipt, moved);
            } 
            Err(e) => {
                error!("event=fs_pair_move_failed error={}", e);
            }
        }
    } else {
        debug!(
            "event=no_matching_yet kind=receipt ref={}",
            receipt.reference_num
        );
    }
}

fn persist_moved_pair(guide: &InssGuide, receipt: &PaymentReceipt, moved: MovedPairPaths) {
    if let Err(e) = persistence::transaction(|tx| {
        persistence::update_path_tx(tx, &guide.path, &moved.guide_path)?;
        persistence::update_path_tx(tx, &receipt.path, &moved.receipt_path)?;
        Ok(())
    }) {
        error!("event=db_transaction_failed error={}", e);
    }
}
