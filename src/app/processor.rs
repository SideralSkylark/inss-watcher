use std::path::PathBuf;
use tracing::{debug, error, info, instrument, warn};

use crate::config::Settings;
use crate::domain::classify::DocumentKind;
use crate::domain::guide::{InssGuide, ParsedGuide};
use crate::domain::receipt::{self, ParsedReceipt, PaymentReceipt};
use crate::domain::{classify, guide};
use crate::infra::fs::{MovedPairPaths, quarantine};
use crate::infra::persistence::StoreOutcome;
use crate::infra::{fs, ocr, pdf, persistence};

#[instrument(
    name = "process_file",
    skip(path, settings),
    fields(
        file = %path.display()
    )
)]
pub fn process_file(path: PathBuf, settings: &Settings) {
    debug!("starting file processing");

    if !pdf::is_candidate(&path) {
        return;
    }

    let mut text = match pdf::extract_text(&path) {
        Ok(t) => t,
        Err(e) => {
            error!(stage = "pdf_extract", error = %e, "failed to extract text from PDF");
            return;
        }
    };

    if text.trim().is_empty() {
        debug!(reason = "empty_pdf", "attempting OCR fallback");

        let img_path = match pdf::pdf_to_img(&path) {
            Ok(p) => p,
            Err(e) => {
                error!(stage = "pdf_to_image", error = %e, "failed to render PDF to image");
                return;
            }
        };

        match ocr::extract_text(&img_path) {
            Ok(ocr_text) => {
                debug!(temp_file = %img_path.display(), chars = ocr_text.len(), "OCR extraction successful");
                text = ocr_text;
            }
            Err(e) => {
                error!(stage = "OCR", temp_file = %img_path.display(), error = %e, "OCR extraction failed");
                return;
            }
        };

        if let Err(e) = std::fs::remove_file(&img_path) {
            warn!(temp_file = %img_path.display(), error = %e, "failed to remove temporary file");
        }
    }

    let kind = classify::classify_doc(&text);

    debug!(text_lenght = text.len(), doc_type = ?kind, "document classified");

    match kind {
        DocumentKind::InssGuide => handle_inss_guide(path, &text, settings),
        DocumentKind::PaymentReceipt => handle_payment_receipt(path, &text, settings),
        DocumentKind::Other => {
            info!(
                    file = %path.display(),
                    reason = "unsupported_type",
                    "document ignored"
            );
        }
    }
}

#[instrument(skip(path, text, settings))]
pub fn handle_inss_guide(path: PathBuf, text: &str, settings: &Settings) {
    debug!("starting guide processing");

    let parsed: ParsedGuide = match guide::parse_guide(&text) {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "parsing failed");
            return;
        }
    };

    let period = format!(
        "{}/{}",
        parsed.reference_period.month, parsed.reference_period.year
    );

    if persistence::guide_exists(&parsed) {
        debug!(
            reference_num = %parsed.reference_num,
            contributor = %parsed.contributor_num,
            period = %period,
            "guide already exists"
        );
        return;
    }

    let guide: InssGuide = (parsed, path).into();

    let outcome = match persistence::store_guide(&guide) {
        Ok(o) => o,
        Err(e) => {
            error!(error = %e, "failure storing guide");
            return;
        }
    };

    if matches!(outcome, StoreOutcome::Inserted) {
        info!(
            reference_num = %guide.reference_num,
            contributor = %guide.contributor_num,
            period = %period,
            "guide stored"
        );
        try_match_guide(&guide, settings);
    }
}

#[instrument(skip(path, text, settings))]
pub fn handle_payment_receipt(path: PathBuf, text: &str, settings: &Settings) {
    debug!("starting receipt processing");

    let parsed: ParsedReceipt = match receipt::parse_receipt(&text) {
        Ok(r) => r,
        Err(e) => {
            warn!(error = %e, "parsing failed");
            return;
        }
    };

    if persistence::receipt_exists(&parsed) {
        debug!("resource already exists");
        return;
    }

    let receipt: PaymentReceipt = (parsed, path).into();

    let outcome = match persistence::store_receipt(&receipt) {
        Ok(o) => o,
        Err(e) => {
            error!(error = %e, "failed storing receipt");
            return;
        }
    };

    if matches!(outcome, StoreOutcome::Inserted) {
        info!(
            reference_num = %receipt.reference_num,
            "receipt stored"
        );
        try_match_receipt(&receipt, settings);
    }
}

#[instrument(name = "try_match_guide", skip(guide, settings))]
fn try_match_guide(guide: &InssGuide, settings: &Settings) {
    let period = format!(
        "{}/{}",
        guide.reference_period.month, guide.reference_period.year
    );

    if let Some(receipt) = persistence::find_matching_receipt(guide) {
        info!(
            reference_num = %guide.reference_num,
            period = %period,
            "matching receipt found"
        );

        match fs::move_pair(&guide, &receipt) {
            Ok(moved) => {
                persist_moved_pair(&guide, &receipt, moved);
            }
            Err(e) => {
                error!(error = %e, "failed to move pair");
            }
        }
    } else {
        debug!(
            reference_num = %guide.reference_num,
            period = %period,
            "no matching receipt found"
        );

        let new_path = match quarantine(&guide.path, &settings.quarantine.quarantine_path) {
            Ok(p) => p,
            Err(e) => {
                error!(error = %e, "failed to quarantine guide");
                return;
            }
        };

        if let Err(e) = persistence::update_path(&guide.path, &new_path) {
            error!(error = %e, "failed to update file's path");
            return;
        };
    }
}

#[instrument(name = "try_match_receipt", skip(receipt, settings))]
fn try_match_receipt(receipt: &PaymentReceipt, settings: &Settings) {
    if let Some(guide) = persistence::find_matching_guide(receipt) {
        info!(
            reference_num = %receipt.reference_num,
            "matching guide found"
        );
        match fs::move_pair(&guide, &receipt) {
            Ok(moved) => {
                persist_moved_pair(&guide, &receipt, moved);
            }
            Err(e) => {
                error!(error = %e, "failed to move pair");
            }
        }
    } else {
        debug!(
            reference_num = %receipt.reference_num,
            "no matching guide found"
        );

        let new_path = match quarantine(&receipt.path, &settings.quarantine.quarantine_path) {
            Ok(p) => p,
            Err(e) => {
                error!(error = %e, "failed to quarantine receipt");
                return;
            }
        };

        if let Err(e) = persistence::update_path(&receipt.path, &new_path) {
            error!( error = %e, "failed to update receipt's path");
            return;
        }
    }
}

#[instrument(
    skip_all,
    fields(
        guide_path = %guide.path.display(),
        receipt_path = %receipt.path.display()
    )
)]
fn persist_moved_pair(guide: &InssGuide, receipt: &PaymentReceipt, moved: MovedPairPaths) {
    debug!("updating resources paths");

    if let Err(e) = persistence::transaction(|tx| {
        persistence::update_path_tx(tx, &guide.path, &moved.guide_path)?;
        persistence::update_path_tx(tx, &receipt.path, &moved.receipt_path)?;
        persistence::mark_matched_tx(tx, &moved.guide_path)?;
        persistence::mark_matched_tx(tx, &moved.receipt_path)?;
        Ok(())
    }) {
        error!(error = %e, "transaction failed");
    }
}
