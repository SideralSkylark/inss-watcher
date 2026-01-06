use std::path::PathBuf;
use log::{debug, info, warn, error};

use crate::domain::classify::DocumentKind;
use crate::domain::guide::InssGuide;
use crate::domain::{classify, guide};
use crate::infra::{fs, ocr, pdf};

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
    debug!("event=document_classified path={:?} kind={:?} raw={}", path, kind, text);

    match kind {
        DocumentKind::InssGuide => {
            handle_inss_guide(path, &text)
        }
        DocumentKind::PaymentReceipt => {
            handle_payment_receipt(path, &text);
        }
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

    debug!(
        "event=inss_guide_metadata_extracted path={:?} month={} year={} contributor={}",
        path,
        guide.reference_period.month,
        guide.reference_period.year,
        guide.contributor_num,
    );

    let out = fs::inss_output_dir(guide.reference_period.month as u32, guide.reference_period.year as u32, &guide.contributor_num);

    if let Err(e) = fs::ensure_dir(&out) {
        error!("event=inss_output_dir_failed path={:?} out_dir={:?} error={}", path, out, e);
        return;
    }

    let filename = match path.file_name() {
        Some(name) => name,
        None => {
            warn!("event=inss_guide_invalid reason=missing_filename path={:?}", path);
            return;
        }
    };

    let mut dest = out;
    dest.push(filename);

    match fs::move_unique(&path, &dest) {
        Ok(_) => info!("event=file_moved kind=inss_guide src={:?} dst={:?}", path, dest),
        Err(e) => error!("event=file_move_failed kind=inss_guide src={:?} dst={:?} error={}", path, dest, e),
    }
}

pub fn handle_payment_receipt(path: PathBuf, text: &str) {
    info!(
        "event=payment_receipt_detected path={:?} raw={}",
        path,
        text
    );
    print!("sucess");
}
