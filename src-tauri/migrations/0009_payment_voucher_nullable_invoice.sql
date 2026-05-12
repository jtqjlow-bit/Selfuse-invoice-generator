-- Allow payment_voucher.invoice_id to be NULL so PVs can be created independently
-- of any invoice (e.g. ad-hoc receipts not tied to a billed document).
-- SQLite cannot drop a NOT NULL constraint in place, so we rebuild the table.

PRAGMA foreign_keys = OFF;

CREATE TABLE payment_voucher_new (
    id TEXT PRIMARY KEY,
    number TEXT NOT NULL UNIQUE,
    invoice_id TEXT REFERENCES invoice(id),
    customer_id TEXT NOT NULL REFERENCES customer(id),
    customer_snapshot TEXT NOT NULL,
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'MYR',
    payment_method TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL
);

INSERT INTO payment_voucher_new
SELECT id, number, invoice_id, customer_id, customer_snapshot, date,
       amount, currency, payment_method, notes, created_at
FROM payment_voucher;

DROP TABLE payment_voucher;

ALTER TABLE payment_voucher_new RENAME TO payment_voucher;

CREATE INDEX idx_payment_voucher_invoice ON payment_voucher(invoice_id);
CREATE INDEX idx_payment_voucher_customer ON payment_voucher(customer_id);
CREATE INDEX idx_payment_voucher_date ON payment_voucher(date);

PRAGMA foreign_keys = ON;
