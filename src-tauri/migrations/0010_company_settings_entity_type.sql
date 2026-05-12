-- Differentiate Company vs Individual entity profiles. Company-only fields
-- (SSM, logo) are kept empty for Individual; Individual requires NRIC.
ALTER TABLE company_settings ADD COLUMN entity_type TEXT NOT NULL DEFAULT 'Company';
ALTER TABLE company_settings ADD COLUMN nric TEXT;
