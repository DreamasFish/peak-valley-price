use chrono::NaiveDate;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::models::{DailyPriceSchedule, Region};

#[derive(Clone)]
pub struct PriceCache {
    cache: Arc<RwLock<HashMap<String, (DailyPriceSchedule, std::time::Instant)>>>,
    ttl: std::time::Duration,
}

impl PriceCache {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: std::time::Duration::from_secs(config.cache.ttl),
        }
    }

    fn make_key(region: &Region, date: NaiveDate) -> String {
        format!("{}:{}", region.province, date)
    }

    pub async fn get(&self, region: &Region, date: NaiveDate) -> Option<DailyPriceSchedule> {
        let key = Self::make_key(region, date);
        let cache = self.cache.read().await;

        if let Some((schedule, inserted_at)) = cache.get(&key) {
            if inserted_at.elapsed() < self.ttl {
                return Some(schedule.clone());
            }
        }

        None
    }

    pub async fn set(&self, schedule: DailyPriceSchedule) {
        let key = Self::make_key(&schedule.region, schedule.date);
        let mut cache = self.cache.write().await;
        cache.insert(key, (schedule, std::time::Instant::now()));
    }

    pub async fn get_or_fetch<F, Fut>(
        &self,
        region: &Region,
        date: NaiveDate,
        fetcher: F,
    ) -> crate::error::Result<DailyPriceSchedule>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = crate::error::Result<DailyPriceSchedule>>,
    {
        if let Some(cached) = self.get(region, date).await {
            return Ok(cached);
        }

        let schedule = fetcher().await?;
        self.set(schedule.clone()).await;
        Ok(schedule)
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        cache.retain(|_, (_, inserted_at)| inserted_at.elapsed() < self.ttl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[tokio::test]
    async fn test_cache_set_get() {
        let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default_config());
        let cache = PriceCache::new(&config);
        let region = Region::jiangsu_wuxi();
        let today = Local::now().date_naive();

        let schedule = DailyPriceSchedule {
            date: today,
            region: region.clone(),
            periods: vec![],
        };

        cache.set(schedule.clone()).await;
        let cached = cache.get(&region, today).await;
        assert!(cached.is_some());
    }
}
