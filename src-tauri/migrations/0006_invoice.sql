CREATE TABLE invoice (
    id TEXT PRIMARY KEY,
    number TEXT NOT NULL UNIQUE,
    customer_id TEXT NOT NULL REFERENCES customer(id),
    customer_snapshot TEXT NOT NULL,
    source_quotation_id TEXT REFERENCES quotation(id),
    issue_date TEXT NOT NULL,
    due_date TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'MYR',
    tax_enabled INTEGER NOT NULL DEFAULT 0 CHECK (tax_enabled IN (0, 1)),
    tax_rate REAL,
    subtotal REAL NOT NULL DEFAULT 0,
    tax_amount REAL NOT NULL DEFAULT 0,
    total REAL NOT NULL DEFAULT 0,
    paid_amount REAL NOT NULL DEFAULT 0,
    payment_methods_snapshot TEXT NOT NULL DEFAULT '[]',
    status TEXT NOT NULL CHECK (status IN ('Draft', 'Sent', 'PartialPaid', 'Paid', 'Overdue', 'Void')),
    notes TEXT,
    terms TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_invoice_customer ON invoice(customer_id);
CREATE INDEX idx_invoice_status ON invoice(status);
CREATE INDEX idx_invoice_due_date ON invoice(due_date);

CREATE TABLE invoice_line_item (
    id TEXT PRIMARY KEY,
    invoice_id TEXT NOT NULL REFERENCES invoice(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    description TEXT NOT NULL,
    quantity REAL NOT NULL,
    unit_price REAL NOT NULL,
    line_total REAL NOT NULL,
    line_currency TEXT NOT NULL DEFAULT 'MYR',
    exchange_rate_to_doc_currency REAL,
    tax_rate REAL,
    discount_rate REAL
);

CREATE INDEX idx_invoice_line_item_invoice ON invoice_line_item(invoice_id);
