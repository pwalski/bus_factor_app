use crate::Result;
use chrono::Utc;
use derive_more::Constructor;
use log::debug;
use log::info;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Constructor, Debug)]
pub struct RateLimit {
    limit: u32,
    remaining: u32,
    reset: i64,
}

#[derive(Constructor)]
pub struct RateLimiter {
    limit: Arc<Mutex<RateLimit>>,
}

impl RateLimiter {
    pub(crate) async fn wait(&self) {
        while let Some(delay) = self.time_to_wait().await {
            info!("Rate limiting wait: {} sec", delay.as_secs());
            tokio::time::sleep(delay).await;
        }
    }

    async fn time_to_wait(&self) -> Option<Duration> {
        let mut rate_limit = self.limit.lock().await;
        if rate_limit.remaining > 0 {
            debug!("Remaining limit {}. Not waiting.", rate_limit.remaining);
            rate_limit.remaining = rate_limit.remaining - 1;
            return None;
        }
        let now = Utc::now().timestamp();
        if rate_limit.reset < now {
            debug!("Old reset. Resetting remaining to limit.");
            //TODO API limit could change so maybe should GET /rate_limit
            rate_limit.remaining = rate_limit.limit - 1;
            return None;
        }
        Some(Duration::new(rate_limit.reset as u64 - now as u64 + 1, 0))
    }

    pub(crate) async fn reset_limiter(&self, headers: &HeaderMap<HeaderValue>) -> crate::Result<()> {
        let mut rate_limit = self.limit.lock().await;
        rate_limit.limit = read_header::<u32>(headers, "x-ratelimit-limit")?;
        // Min `remaining` because in case of parallel requests late response may arrive with old `remaining`
        rate_limit.remaining = std::cmp::min(
            read_header::<u32>(headers, "x-ratelimit-remaining")?,
            rate_limit.remaining,
        );
        // Max `reset` because in case of parallel requests late response may arrive with old `reset`
        rate_limit.reset = std::cmp::max(read_header::<i64>(headers, "x-ratelimit-reset")?, rate_limit.reset);
        debug!("Updated limits: {:?}", rate_limit);
        Ok(())
    }
}

fn read_header<T>(headers: &HeaderMap<HeaderValue>, header: &str) -> Result<T>
where
    T: FromStr,
    crate::Error: From<<T as FromStr>::Err>,
{
    let header = headers
        .get(header)
        .ok_or_else(|| format!("Header {} not found", header.to_string()))
        .map(HeaderValue::to_str)??;
    Ok(header.parse::<T>()?)
}

#[tokio::test]
async fn wait_test() -> anyhow::Result<()> {
    let reset = Utc::now().timestamp() + 1;
    let limit = RateLimit::new(3, 1, reset);
    let limiter = RateLimiter::new(Arc::new(Mutex::new(limit)));
    limiter.wait().await;
    assert_eq!(
        Utc::now().timestamp(),
        reset - 1,
        "Limiter should not wait with remaining set to 1"
    );

    limiter.wait().await;
    assert_eq!(Utc::now().timestamp(), reset + 1, "Limiter should wait 1s");

    let then = Utc::now().timestamp();
    limiter.wait().await;
    limiter.wait().await;
    assert_eq!(
        Utc::now().timestamp(),
        then,
        "Remaining should be reset after reaching limit, so no wait."
    );

    let reset = then + 1;
    let mut headers = HeaderMap::new();
    headers.insert("x-ratelimit-limit", HeaderValue::from_str("3")?);
    headers.insert("x-ratelimit-remaining", HeaderValue::from_str("2")?);
    headers.insert("x-ratelimit-reset", HeaderValue::from_str(&format!("{}", reset))?);
    limiter.reset_limiter(&headers).await?;
    assert_eq!(
        reset,
        Utc::now().timestamp() + 1,
        "Reset has been reset, but remaining arriving in header has been ignored, so limiter should wait."
    );

    Ok(())
}
