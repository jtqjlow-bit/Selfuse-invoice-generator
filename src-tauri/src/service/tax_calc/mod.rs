pub struct LineForTotals {
    pub quantity: f64,
    pub unit_price: f64,
}

pub struct Totals {
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total: f64,
}

/// Pure totals calculation shared by quotation and invoice.
///
/// v1 semantics: line-level tax and discount are ignored; only document-level
/// `tax_enabled` + `tax_rate` is honoured. When line-level adjustments are
/// added, fold them in here so all callers benefit.
pub fn document_totals(
    lines: &[LineForTotals],
    tax_enabled: bool,
    tax_rate: Option<f64>,
) -> Totals {
    let subtotal: f64 = lines.iter().map(|l| l.quantity * l.unit_price).sum();
    let tax_amount = if tax_enabled {
        subtotal * tax_rate.unwrap_or(0.0)
    } else {
        0.0
    };
    Totals {
        subtotal,
        tax_amount,
        total: subtotal + tax_amount,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_lines_is_zero() {
        let t = document_totals(&[], false, None);
        assert_eq!(t.subtotal, 0.0);
        assert_eq!(t.tax_amount, 0.0);
        assert_eq!(t.total, 0.0);
    }

    #[test]
    fn no_tax_when_disabled() {
        let t = document_totals(
            &[LineForTotals {
                quantity: 2.0,
                unit_price: 100.0,
            }],
            false,
            Some(0.06),
        );
        assert_eq!(t.subtotal, 200.0);
        assert_eq!(t.tax_amount, 0.0);
        assert_eq!(t.total, 200.0);
    }

    #[test]
    fn applies_tax_when_enabled() {
        let t = document_totals(
            &[
                LineForTotals { quantity: 1.0, unit_price: 3000.0 },
                LineForTotals { quantity: 2.0, unit_price: 500.0 },
            ],
            true,
            Some(0.06),
        );
        assert!((t.subtotal - 4000.0).abs() < 1e-9);
        assert!((t.tax_amount - 240.0).abs() < 1e-9);
        assert!((t.total - 4240.0).abs() < 1e-9);
    }

    #[test]
    fn null_tax_rate_treats_as_zero() {
        let t = document_totals(
            &[LineForTotals { quantity: 1.0, unit_price: 100.0 }],
            true,
            None,
        );
        assert_eq!(t.tax_amount, 0.0);
        assert_eq!(t.total, 100.0);
    }
}
