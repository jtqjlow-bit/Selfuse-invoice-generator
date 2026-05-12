-- Replace the singleton company_settings table with a multi-row business_profile.
-- Each Quotation / Invoice / Payment Voucher picks the profile it was issued from.
-- Existing company_settings data is discarded (per user instruction — it was test data).

DROP TABLE IF EXISTS company_settings;

CREATE TABLE business_profile (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    name TEXT NOT NULL,
    address TEXT,
    email TEXT,
    phone TEXT,
    ssm_no TEXT,
    nric TEXT,
    sst_no TEXT,
    logo_path TEXT,
    qr_path TEXT,
    bank_accounts TEXT NOT NULL DEFAULT '[]',
    enabled_payment_methods TEXT NOT NULL DEFAULT '[]',
    default_tax_rate REAL,
    default_quotation_valid_days INTEGER NOT NULL DEFAULT 30,
    default_invoice_due_days INTEGER NOT NULL DEFAULT 30,
    data_dir TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

ALTER TABLE quotation ADD COLUMN business_profile_id TEXT REFERENCES business_profile(id);
ALTER TABLE invoice ADD COLUMN business_profile_id TEXT REFERENCES business_profile(id);
ALTER TABLE payment_voucher ADD COLUMN business_profile_id TEXT REFERENCES business_profile(id);

CREATE INDEX idx_quotation_business_profile ON quotation(business_profile_id);
CREATE INDEX idx_invoice_business_profile ON invoice(business_profile_id);
CREATE INDEX idx_payment_voucher_business_profile ON payment_voucher(business_profile_id);
