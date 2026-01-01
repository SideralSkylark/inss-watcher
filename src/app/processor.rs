use std::path::PathBuf;
use log::{debug, info, warn, error};

use crate::domain::inss;
use crate::infra::{pdf, fs};

pub fn process_file(path: PathBuf) {
    info!("event=file_processing_started path={:?}", path);

    let text = match pdf::extract_text(&path) {
        Ok(t) => t,
        Err(e) => {
            error!("event=processing_failed stage=pdf_extract path={:?} error={}", path, e);
            return;
        }
    };

    if text.len() == 0 {
        warn!("event=processing_failed path={:?} error=pdf_empty", path);
        return;
    }

    let kind = inss::classify_document(&text);
    debug!("event=document_classified path={:?} kind={:?}", path, kind);

    match kind {
        inss::DocumentKind::InssGuide => {
            handle_inss_guide(path, &text)
        }
        inss::DocumentKind::PaymentReceipt => {
            handle_payment_receipt(path, &text);
        }
        inss::DocumentKind::Other => {
            info!("event=document_ignored reason=unsupported_type path={:?}", path);
        }
    }
}

pub fn handle_inss_guide(path: PathBuf, text: &str) {
    info!("event=inss_guide_processing_started path={:?}", path);

    let (month, year) = match inss::extract_reference_date(&text) {
        Some(d) => d,
        None => {
            warn!("event=inss_guide_invalid reason=missing_reference_date path={:?}", path);
            return;
        }
    };

    let contributor_num = match inss::extract_contributor_num(&text) {
        Some(num) => num,
        None => {
            warn!("event=inss_guide_invalid reason=missing_contributor path={:?}", path);
            return;
        }
    };

    debug!(
        "event=inss_guide_metadata_extracted path={:?} month={} year={} contributor={}",
        path,
        month,
        year,
        contributor_num
    );

    let out = fs::inss_output_dir(month, year, &contributor_num);

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
        "event=payment_receipt_detected path={:?}",
        path
    );
    todo!("implement receipt logic");
}
