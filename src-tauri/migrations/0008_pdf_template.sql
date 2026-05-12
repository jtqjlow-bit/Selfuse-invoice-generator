CREATE TABLE pdf_template (
    id TEXT PRIMARY KEY,
    doc_type TEXT NOT NULL CHECK (doc_type IN ('Quotation', 'Invoice', 'PaymentVoucher')),
    name TEXT NOT NULL,
    type_ TEXT NOT NULL CHECK (type_ IN ('Preset', 'Custom')),
    file_path TEXT NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_pdf_template_doc_type ON pdf_template(doc_type);
CREATE INDEX idx_pdf_template_type ON pdf_template(type_);

-- Three preset rows. file_path stores a "preset:<key>" sentinel that
-- pdf_template::service::get_renderable resolves against the compiled-in
-- HTML constants.
INSERT INTO pdf_template (id, doc_type, name, type_, file_path, config_json, created_at, updated_at)
VALUES
  ('preset-quotation-default', 'Quotation', '默认 Quotation 模板', 'Preset', 'preset:quotation_default', '{}',
   strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  ('preset-invoice-default', 'Invoice', '默认 Invoice 模板', 'Preset', 'preset:invoice_default', '{}',
   strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  ('preset-pv-default', 'PaymentVoucher', '默认 Payment Voucher 模板', 'Preset', 'preset:pv_default', '{}',
   strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
