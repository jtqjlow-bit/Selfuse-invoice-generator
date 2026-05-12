CREATE TABLE quotation (
    id TEXT PRIMARY KEY,
    number TEXT NOT NULL UNIQUE,
    customer_id TEXT NOT NULL REFERENCES customer(id),
    customer_snapshot TEXT NOT NULL,
    issue_date TEXT NOT NULL,
    valid_until TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'MYR',
    tax_enabled INTEGER NOT NULL DEFAULT 0 CHECK (tax_enabled IN (0, 1)),
    tax_rate REAL,
    subtotal REAL NOT NULL DEFAULT 0,
    tax_amount REAL NOT NULL DEFAULT 0,
    total REAL NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('Draft', 'Sent', 'Accepted', 'Rejected', 'Expired')),
    converted_invoice_id TEXT,
    notes TEXT,
    terms TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_quotation_customer ON quotation(customer_id);
CREATE INDEX idx_quotation_status ON quotation(status);

CREATE TABLE quotation_line_item (
    id TEXT PRIMARY KEY,
    quotation_id TEXT NOT NULL REFERENCES quotation(id) ON DELETE CASCADE,
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

CREATE INDEX idx_quotation_line_item_quotation ON quotation_line_item(quotation_id);
