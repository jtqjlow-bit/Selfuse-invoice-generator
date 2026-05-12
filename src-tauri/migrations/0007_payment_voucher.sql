CREATE TABLE payment_voucher (
    id TEXT PRIMARY KEY,
    number TEXT NOT NULL UNIQUE,
    invoice_id TEXT NOT NULL REFERENCES invoice(id),
    customer_id TEXT NOT NULL REFERENCES customer(id),
    customer_snapshot TEXT NOT NULL,
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'MYR',
    payment_method TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_payment_voucher_invoice ON payment_voucher(invoice_id);
CREATE INDEX idx_payment_voucher_customer ON payment_voucher(customer_id);
CREATE INDEX idx_payment_voucher_date ON payment_voucher(date);
