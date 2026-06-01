-- Exchange rate cache owned by service::currency.
-- One row per (base, target) pair; rate fetched from exchangerate.host,
-- refreshed on demand or when older than 24h.
CREATE TABLE exchange_rate_cache (
    base TEXT NOT NULL,
    target TEXT NOT NULL,
    rate REAL NOT NULL,
    fetched_at TEXT NOT NULL,
    PRIMARY KEY (base, target)
);
