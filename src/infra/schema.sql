CREATE TABLE IF NOT EXISTS documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    doc_type TEXT NOT NULL,            
    status TEXT NOT NULL,             

    reference_num TEXT NOT NULL,
    contributor_num TEXT,
    ref_month INTEGER,
    ref_year INTEGER,

    amount_cents INTEGER NOT NULL,
    payment_date TEXT,

    path TEXT NOT NULL,

    matched_with INTEGER,
    created_at TEXT NOT NULL,

    UNIQUE (
        doc_type,
        reference_num,
        amount_cents,
        ref_month,
        ref_year,
        payment_date
    )
);
