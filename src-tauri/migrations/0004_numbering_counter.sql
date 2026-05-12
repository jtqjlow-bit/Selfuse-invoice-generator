CREATE TABLE numbering_counter (
    doc_type TEXT NOT NULL CHECK (doc_type IN ('Quotation', 'Invoice', 'PaymentVoucher')),
    year INTEGER NOT NULL,
    last_seq INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (doc_type, year)
);
