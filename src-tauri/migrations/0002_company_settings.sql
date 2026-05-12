CREATE TABLE company_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    company_name TEXT NOT NULL DEFAULT '',
    address TEXT,
    email TEXT,
    phone TEXT,
    ssm_no TEXT,
    sst_no TEXT,
    logo_path TEXT,
    qr_path TEXT,
    bank_accounts TEXT NOT NULL DEFAULT '[]',
    enabled_payment_methods TEXT NOT NULL DEFAULT '[]',
    default_tax_rate REAL,
    default_quotation_valid_days INTEGER NOT NULL DEFAULT 30,
    default_invoice_due_days INTEGER NOT NULL DEFAULT 30,
    data_dir TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO company_settings (id, updated_at)
VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
