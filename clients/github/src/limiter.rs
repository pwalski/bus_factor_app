use chrono::Utc;
use clients::api::Result;
use derive_more::Constructor;
use log::debug;
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
    pub async fn wait(&self) {
        while let Some(delay) = self.time_to_wait().await {
            debug!("Rate limiting wait: {}", delay.as_secs());
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

    pub async fn reset_limiter(&self, headers: &HeaderMap<HeaderValue>) -> Result<()> {
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
    clients::api::Error: From<<T as FromStr>::Err>,
{
    let header = headers
        .get(header)
        .ok_or_else(|| format!("Header {} not found", header.to_string()))
        .map(HeaderValue::to_str)??;
    Ok(header.parse::<T>()?)
}
