use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};

use crate::error::{AppError, AppResult};
use crate::infra::Db;

use super::repository;
use super::types::ExchangeRate;

const API_LATEST: &str = "https://api.exchangerate.host/latest";
const CACHE_TTL_HOURS: i64 = 24;

#[derive(serde::Deserialize)]
struct LatestResponse {
    rates: HashMap<String, f64>,
}

fn normalize(code: &str) -> AppResult<String> {
    let c = code.trim().to_uppercase();
    if c.len() != 3 || !c.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Err(AppError::Validation(format!("非法货币代码: {code}")));
    }
    Ok(c)
}

fn is_fresh(fetched_at: &str) -> bool {
    match DateTime::parse_from_rfc3339(fetched_at) {
        Ok(t) => {
            Utc::now().signed_duration_since(t.with_timezone(&Utc)) < Duration::hours(CACHE_TTL_HOURS)
        }
        Err(_) => false,
    }
}

fn fetch_rate(from: &str, to: &str) -> AppResult<f64> {
    let url = format!("{API_LATEST}?base={from}&symbols={to}");
    let resp: LatestResponse = ureq::get(&url)
        .call()
        .map_err(|e| AppError::Internal(format!("汇率请求失败: {e}")))?
        .into_json()
        .map_err(|e| AppError::Internal(format!("汇率响应解析失败: {e}")))?;
    resp.rates
        .get(to)
        .copied()
        .ok_or_else(|| AppError::Internal(format!("汇率响应缺少目标货币 {to}")))
}

/// Latest rate from `from` to `to`. Returns cached value when fresh (<24h),
/// otherwise fetches from exchangerate.host and updates the cache.
pub fn get_rate(db: &Db, from: &str, to: &str) -> AppResult<f64> {
    let from = normalize(from)?;
    let to = normalize(to)?;
    if from == to {
        return Ok(1.0);
    }
    if let Some(cached) = db.with_conn(|c| repository::get_cached(c, &from, &to))? {
        if is_fresh(&cached.fetched_at) {
            return Ok(cached.rate);
        }
    }
    let rate = fetch_rate(&from, &to)?;
    let now = Utc::now().to_rfc3339();
    db.transaction(|tx| repository::upsert(tx, &from, &to, rate, &now))?;
    Ok(rate)
}

pub fn convert(db: &Db, amount: f64, from: &str, to: &str) -> AppResult<f64> {
    let rate = get_rate(db, from, to)?;
    Ok(amount * rate)
}

/// Force-refresh every cached pair from the network. Returns how many were refreshed.
pub fn refresh(db: &Db) -> AppResult<u32> {
    let pairs = db.with_conn(|c| repository::list_all(c))?;
    let mut count = 0;
    for p in pairs {
        let rate = fetch_rate(&p.base, &p.target)?;
        let now = Utc::now().to_rfc3339();
        db.transaction(|tx| repository::upsert(tx, &p.base, &p.target, rate, &now))?;
        count += 1;
    }
    Ok(count)
}

pub fn list_cached(db: &Db) -> AppResult<Vec<ExchangeRate>> {
    db.with_conn(|c| repository::list_all(c))
}
