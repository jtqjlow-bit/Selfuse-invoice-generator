-- Slice B: multi-QR (typed) + per-Invoice payment picker.
--
-- BusinessProfile now stores a `qrs` JSON array (each entry: id, kind, label,
-- file_path). The legacy `qr_path` column stays but is unused.
--
-- Each Invoice records three independent multi-selects against the profile's
-- inventory at render time:
--   selected_bank_account_ids — references BankAccount.id (UUID) inside profile
--   selected_qr_ids           — references Qr.id (UUID) inside profile
--   selected_static_methods   — free strings like "Cash" / "Cheque" / "FPX"

ALTER TABLE business_profile ADD COLUMN qrs TEXT NOT NULL DEFAULT '[]';

ALTER TABLE invoice ADD COLUMN selected_bank_account_ids TEXT NOT NULL DEFAULT '[]';
ALTER TABLE invoice ADD COLUMN selected_qr_ids TEXT NOT NULL DEFAULT '[]';
ALTER TABLE invoice ADD COLUMN selected_static_methods TEXT NOT NULL DEFAULT '[]';
