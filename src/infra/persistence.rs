use std::sync::{OnceLock, Mutex};

use anyhow::{Result, Context};
use rusqlite::{Connection, params};
use chrono::NaiveDate;

use crate::domain::guide::{InssGuide, ParsedGuide, ReferencePeriod};
use crate::domain::money::Money;
use crate::domain::matcher;
use crate::domain::receipt::{ParsedReceipt, PaymentReceipt};

pub enum StoreOutcome {
    Inserted,
    AlreadyExists,
}

static DB: OnceLock<Mutex<Connection>> = OnceLock::new();

pub fn init(db_path: &str) -> Result<()> {
    let conn = Connection::open(db_path)
        .context("failed to open sqlite database")?;

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

pub fn guide_exists(guide: &ParsedGuide) -> bool {
    let c = conn();
    let mut stmt = c.prepare(
        "SELECT 1 FROM documents WHERE doc_type='guide' AND reference_num=?1 AND contributor_num=?2 AND ref_month=?3 AND ref_year=?4 LIMIT 1"
    ).unwrap();

    let exists = stmt.exists(params![
        guide.reference_num,
        guide.contributor_num,
        guide.reference_period.month,
        guide.reference_period.year
    ]).unwrap_or(false);

    exists
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

    Ok(if rows == 0 { StoreOutcome::AlreadyExists } else { StoreOutcome::Inserted })
}


pub fn receipt_exists(receipt: &ParsedReceipt) -> bool {
    let c = conn();
    let mut stmt = c.prepare(
        "SELECT 1 FROM documents WHERE doc_type='receipt' AND reference_num=?1 AND amount_cents=?2 LIMIT 1"
    ).unwrap();

    stmt.exists(params![receipt.reference_num, receipt.amount.cents]).unwrap_or(false)
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

    Ok(if rows == 0 { StoreOutcome::AlreadyExists } else { StoreOutcome::Inserted })
}

pub fn find_matching_receipt(guide: &InssGuide) -> Option<PaymentReceipt> {
    let c = conn();
    let mut stmt = c.prepare(
        "SELECT path, reference_num, amount_cents, payment_date
         FROM documents
         WHERE doc_type='receipt' AND reference_num=?1"
    ).unwrap();

    let mut rows = stmt.query(params![guide.reference_num]).unwrap();

    while let Some(row) = rows.next().unwrap() {
        let path: String = row.get(0).unwrap();
        let reference_num: String = row.get(1).unwrap();
        let amount_cents: i64 = row.get(2).unwrap();
        let payment_date_str: String = row.get(3).unwrap();
        let payment_date = NaiveDate::parse_from_str(&payment_date_str, "%Y-%m-%d").unwrap();

        if guide.amount.cents == amount_cents && matcher::within_period(guide.reference_period, payment_date) {
            return Some(PaymentReceipt {
                reference_num,
                payment_date,
                amount: Money { cents: amount_cents },
                path: path.into(),
            });
        }
    }

    None
}

pub fn find_matching_guide(receipt: &PaymentReceipt) -> Option<InssGuide> {
    let c = conn();
    let mut stmt = c.prepare(
        "SELECT path, reference_num, contributor_num, ref_month, ref_year, amount_cents
         FROM documents
         WHERE doc_type='guide' AND reference_num=?1"
    ).unwrap();

    let mut rows = stmt.query(params![receipt.reference_num]).unwrap();

    while let Some(row) = rows.next().unwrap() {
        let path: String = row.get(0).unwrap();
        let reference_num: String = row.get(1).unwrap();
        let contributor_num: String = row.get(2).unwrap();
        let month: u8 = row.get::<_, i64>(3).unwrap() as u8;
        let year: u16 = row.get::<_, i64>(4).unwrap() as u16;
        let amount_cents: i64 = row.get::<_, i64>(5).unwrap();

        if receipt.amount.cents == amount_cents && matcher::within_period(ReferencePeriod { month, year }, receipt.payment_date) {
            return Some(InssGuide {
                reference_num,
                contributor_num,
                reference_period: ReferencePeriod { month, year },
                amount: Money { cents: amount_cents },
                path: path.into(),
            });
        }
    }

    None
}
