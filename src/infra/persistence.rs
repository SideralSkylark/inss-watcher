use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use anyhow::{Context, Result};
use chrono::NaiveDate;
use rusqlite::{Connection, Transaction, params};

use crate::app::orchestrator::{StatusResponse, UnmatchedArtifact};
use crate::domain::guide::{InssGuide, ParsedGuide, ReferencePeriod};
use crate::domain::matcher;
use crate::domain::money::Money;
use crate::domain::receipt::{ParsedReceipt, PaymentReceipt};

pub enum StoreOutcome {
    Inserted,
    AlreadyExists,
}

static DB: OnceLock<Mutex<Connection>> = OnceLock::new();

pub fn init(db_path: &PathBuf) -> Result<()> {
    let conn = Connection::open(db_path).context("failed to open sqlite database")?;

    conn.execute_batch(include_str!("schema.sql"))
        .context("failed to initialize database schema")?;

    DB.set(Mutex::new(conn))
        .map_err(|_| anyhow::anyhow!("database already initialized"))?;

    Ok(())
}

fn conn() -> std::sync::MutexGuard<'static, Connection> {
    DB.get()
        .expect("database not initialized: call persistence::init() first")
        .lock()
        .expect("database mutex poisoned")
}

pub fn transaction<F, T>(f: F) -> anyhow::Result<T>
where
    F: FnOnce(&Transaction) -> anyhow::Result<T>,
{
    let mut c = conn();

    let tx = c.transaction().context("failed to start transaction")?;

    match f(&tx) {
        Ok(result) => {
            tx.commit().context("failed to commit transaction")?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback().ok(); // best effort
            Err(e)
        }
    }
}

/// checks for another guide with the same reference number, contributor_num, and reference
/// period(month & year)
pub fn guide_exists(guide: &ParsedGuide) -> anyhow::Result<bool> {
    let c = conn();
    let mut stmt = c.prepare(
        "SELECT 1 FROM documents WHERE doc_type='guide' AND reference_num=?1 AND contributor_num=?2 AND ref_month=?3 AND ref_year=?4 LIMIT 1"
    )?;

    let exists = stmt.exists(params![
        guide.reference_num,
        guide.contributor_num,
        guide.reference_period.month,
        guide.reference_period.year
    ])?;

    Ok(exists)
}

/// checks for another receipt with the same reference number and amount
pub fn receipt_exists(receipt: &ParsedReceipt) -> anyhow::Result<bool> {
    let c = conn();
    let mut stmt = c.prepare(
        "SELECT 1 FROM documents WHERE doc_type='receipt' AND reference_num=?1 AND amount_cents=?2 LIMIT 1"
    )?;

    Ok(stmt.exists(params![receipt.reference_num, receipt.amount.cents])?)
}

pub fn store_guide(guide: &InssGuide) -> anyhow::Result<StoreOutcome> {
    let c = conn();

    let rows = c.execute(
        r#"
        INSERT OR IGNORE INTO documents (
            doc_type, status, reference_num, contributor_num,
            ref_month, ref_year, amount_cents, path, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))
        "#,
        params![
            "guide",
            "stored",
            guide.reference_num,
            guide.contributor_num,
            guide.reference_period.month,
            guide.reference_period.year,
            guide.amount.cents,
            guide.path.to_string_lossy()
        ],
    )?;

    Ok(if rows == 0 {
        StoreOutcome::AlreadyExists
    } else {
        StoreOutcome::Inserted
    })
}

pub fn store_receipt(receipt: &PaymentReceipt) -> anyhow::Result<StoreOutcome> {
    let c = conn();

    let rows = c.execute(
        r#"
        INSERT OR IGNORE INTO documents (
            doc_type, status, reference_num, contributor_num,
            ref_month, ref_year, amount_cents, payment_date, path, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))
        "#,
        params![
            "receipt",
            "stored",
            receipt.reference_num,
            "",
            0,
            0,
            receipt.amount.cents,
            receipt.payment_date.format("%Y-%m-%d").to_string(),
            receipt.path.to_string_lossy()
        ],
    )?;

    Ok(if rows == 0 {
        StoreOutcome::AlreadyExists
    } else {
        StoreOutcome::Inserted
    })
}

pub fn find_matching_receipt(guide: &InssGuide) -> anyhow::Result<Option<PaymentReceipt>> {
    let c = conn();
    let mut stmt = c.prepare(
        "SELECT path, reference_num, amount_cents, payment_date
         FROM documents
         WHERE doc_type='receipt' AND reference_num=?1",
    )?;

    let mut rows = stmt.query(params![guide.reference_num])?;

    while let Some(row) = rows.next()? {
        let path: String = row.get(0)?;
        let reference_num: String = row.get(1)?;
        let amount_cents: i64 = row.get(2)?;
        let payment_date_str: String = row.get(3)?;
        let payment_date = NaiveDate::parse_from_str(&payment_date_str, "%Y-%m-%d")
            .map_err(|e| anyhow::anyhow!("invalid date in database: {}", e))?;

        if guide.amount.cents == amount_cents
            && matcher::within_period(guide.reference_period, payment_date)
        {
            return Ok(Some(PaymentReceipt {
                reference_num,
                payment_date,
                amount: Money {
                    cents: amount_cents,
                },
                path: path.into(),
            }));
        }
    }

    Ok(None)
}

pub fn find_matching_guide(receipt: &PaymentReceipt) -> anyhow::Result<Option<InssGuide>> {
    let c = conn();
    let mut stmt = c.prepare(
        "SELECT path, reference_num, contributor_num, ref_month, ref_year, amount_cents
         FROM documents
         WHERE doc_type='guide' AND reference_num=?1",
    )?;

    let mut rows = stmt.query(params![receipt.reference_num])?;

    while let Some(row) = rows.next()? {
        let path: String = row.get(0)?;
        let reference_num: String = row.get(1)?;
        let contributor_num: String = row.get(2)?;
        let month: u8 = row.get::<_, i64>(3)? as u8;
        let year: u16 = row.get::<_, i64>(4)? as u16;
        let amount_cents: i64 = row.get::<_, i64>(5)?;

        if receipt.amount.cents == amount_cents
            && matcher::within_period(ReferencePeriod { month, year }, receipt.payment_date)
        {
            return Ok(Some(InssGuide {
                reference_num,
                contributor_num,
                reference_period: ReferencePeriod { month, year },
                amount: Money {
                    cents: amount_cents,
                },
                path: path.into(),
            }));
        }
    }

    Ok(None)
}

pub fn mark_matched_tx(tx: &Transaction, path: &Path) -> anyhow::Result<()> {
    tx.execute(
        "UPDATE documents SET status = 'matched' WHERE path = ?1",
        params![path.to_string_lossy()],
    )?;
    Ok(())
}

pub fn query_status(queue_depth: usize) -> anyhow::Result<StatusResponse> {
    let c = conn();

    let matched: usize = c.query_row(
        "SELECT COUNT(*) FROM documents WHERE status = 'matched'",
        [],
        |row| row.get::<_, i64>(0),
    )? as usize;

    let mut stmt = c.prepare(
        "SELECT doc_type, reference_num, ref_month, ref_year, path
         FROM documents
         WHERE status = 'stored'", // 'stored' = unmatched
    )?;

    let unmatched = stmt
        .query_map([], |row| {
            let doc_type: String = row.get(0)?;
            let reference_num: String = row.get(1)?;
            let month: i64 = row.get(2)?;
            let year: i64 = row.get(3)?;
            let path: String = row.get(4)?;
            Ok((doc_type, reference_num, month, year, path))
        })?
        .filter_map(|r| r.ok())
        .map(
            |(kind, reference_num, month, year, path)| UnmatchedArtifact {
                period: if month > 0 {
                    format!("{:02}/{}", month, year)
                } else {
                    String::new()
                },
                kind,
                reference_num,
                path,
            },
        )
        .collect::<Vec<_>>();

    Ok(StatusResponse {
        queue_depth,
        matched,
        unmatched,
    })
}

pub fn update_path(old_path: &Path, new_path: &Path) -> anyhow::Result<()> {
    let c = conn();

    c.execute(
        "UPDATE documents SET path=?1 WHERE path=?2",
        params![new_path.to_string_lossy(), old_path.to_string_lossy()],
    )?;

    Ok(())
}

/// updates a resource's path using a database transaction
pub fn update_path_tx(
    tx: &rusqlite::Transaction,
    old_path: &Path,
    new_path: &Path,
) -> anyhow::Result<()> {
    tx.execute(
        "UPDATE documents SET path=?1 WHERE path=?2",
        params![new_path.to_string_lossy(), old_path.to_string_lossy()],
    )?;

    Ok(())
}
