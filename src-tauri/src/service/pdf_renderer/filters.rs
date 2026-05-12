//! Custom Tera filters used by PDF templates.
//!
//! Per CLAUDE.md §8:
//!   - `currency(code=...)`  — format a number with thousand separators + 2 decimals, prefixed by the code
//!   - `date(format=...)`    — reformat an ISO date string with `chrono::format::strftime`
//!   - `nl2br`               — replace newline characters with `<br>`
use std::collections::HashMap;

use tera::{Result, Tera, Value};

pub fn register_all(tera: &mut Tera) {
    tera.register_filter("currency", currency_filter);
    tera.register_filter("date", date_filter);
    tera.register_filter("nl2br", nl2br_filter);
}

fn currency_filter(value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
    let amount = value.as_f64().ok_or_else(|| {
        tera::Error::msg(format!("currency: expected number, got {value:?}"))
    })?;
    let code = args
        .get("code")
        .and_then(|v| v.as_str())
        .unwrap_or("MYR");
    Ok(Value::String(format_currency(amount, code)))
}

fn format_currency(amount: f64, code: &str) -> String {
    let abs = amount.abs();
    let int_part = abs.trunc() as u64;
    let frac_part = ((abs - abs.trunc()) * 100.0).round() as u64;
    let int_str = with_thousands_separator(int_part);
    let sign = if amount < 0.0 { "-" } else { "" };
    format!("{code} {sign}{int_str}.{frac_part:02}")
}

fn with_thousands_separator(n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(b as char);
    }
    out
}

fn date_filter(value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg(format!("date: expected string, got {value:?}")))?;
    let format = args
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("%Y-%m-%d");

    // Accept either YYYY-MM-DD or full RFC3339; pick the right parser.
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(Value::String(d.format(format).to_string()));
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(Value::String(dt.format(format).to_string()));
    }
    Err(tera::Error::msg(format!(
        "date: cannot parse {s} as YYYY-MM-DD or RFC3339"
    )))
}

fn nl2br_filter(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg(format!("nl2br: expected string, got {value:?}")))?;
    // Escape HTML and convert newlines.
    let escaped = tera::escape_html(s);
    Ok(Value::String(escaped.replace('\n', "<br>")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tera() -> Tera {
        let mut t = Tera::default();
        register_all(&mut t);
        t
    }

    #[test]
    fn currency_basic() {
        let mut t = tera();
        let r = t
            .render_str("{{ 1234.5 | currency(code=\"USD\") }}", &tera::Context::new())
            .unwrap();
        assert_eq!(r, "USD 1,234.50");
    }

    #[test]
    fn currency_thousands_separator() {
        let mut t = tera();
        let r = t
            .render_str("{{ 1234567.89 | currency(code=\"MYR\") }}", &tera::Context::new())
            .unwrap();
        assert_eq!(r, "MYR 1,234,567.89");
    }

    #[test]
    fn currency_zero_and_small() {
        let mut t = tera();
        let r = t
            .render_str("{{ 0 | currency(code=\"MYR\") }}", &tera::Context::new())
            .unwrap();
        assert_eq!(r, "MYR 0.00");
        let r2 = t
            .render_str("{{ 0.05 | currency(code=\"MYR\") }}", &tera::Context::new())
            .unwrap();
        assert_eq!(r2, "MYR 0.05");
    }

    #[test]
    fn currency_negative() {
        let mut t = tera();
        let r = t
            .render_str("{{ -12.34 | currency(code=\"MYR\") }}", &tera::Context::new())
            .unwrap();
        assert_eq!(r, "MYR -12.34");
    }

    #[test]
    fn date_iso_to_human() {
        let mut t = tera();
        let r = t
            .render_str(
                "{{ \"2026-05-12\" | date(format=\"%d %b %Y\") }}",
                &tera::Context::new(),
            )
            .unwrap();
        assert_eq!(r, "12 May 2026");
    }

    #[test]
    fn date_rfc3339_input() {
        let mut t = tera();
        let r = t
            .render_str(
                "{{ \"2026-05-12T10:30:00+08:00\" | date(format=\"%Y-%m-%d\") }}",
                &tera::Context::new(),
            )
            .unwrap();
        assert_eq!(r, "2026-05-12");
    }

    #[test]
    fn nl2br_inserts_breaks_and_escapes_html() {
        let mut t = tera();
        let ctx = tera::Context::from_value(json!({ "s": "line1\nline2 <evil>" })).unwrap();
        let r = t.render_str("{{ s | nl2br | safe }}", &ctx).unwrap();
        assert_eq!(r, "line1<br>line2 &lt;evil&gt;");
    }
}
