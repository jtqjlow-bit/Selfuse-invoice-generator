CREATE TABLE customer (
    id TEXT PRIMARY KEY,
    type_ TEXT NOT NULL CHECK (type_ IN ('Company', 'Individual')),
    name TEXT NOT NULL,
    contact_person TEXT,
    email TEXT,
    phone TEXT,
    address TEXT,
    ssm_no TEXT,
    nric TEXT,
    tax_no TEXT,
    notes TEXT,
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_customer_archived ON customer(archived);
CREATE INDEX idx_customer_name ON customer(name);
