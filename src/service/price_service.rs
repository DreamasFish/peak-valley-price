use chrono::{Local, NaiveDate, Timelike};

use crate::api::traits::PriceProvider;
use crate::config::AppConfig;
use crate::error::Result;
use crate::models::{DailyPriceSchedule, PriceTier, Region};
use crate::service::cache::PriceCache;

pub struct PriceService {
    provider: Box<dyn PriceProvider>,
    cache: PriceCache,
}

impl PriceService {
    pub fn new(provider: Box<dyn PriceProvider>, config: &AppConfig) -> Self {
        Self {
            provider,
            cache: PriceCache::new(config),
        }
    }

    pub async fn current_price(&self, region: &Region) -> Result<DailyPriceSchedule> {
        let today = Local::now().date_naive();
        self.get_schedule(region, today).await
    }

    pub async fn today_prices(&self, region: &Region) -> Result<DailyPriceSchedule> {
        self.current_price(region).await
    }

    pub async fn prices_for_date(
        &self,
        region: &Region,
        date: NaiveDate,
    ) -> Result<DailyPriceSchedule> {
        self.get_schedule(region, date).await
    }

    async fn get_schedule(
        &self,
        region: &Region,
        date: NaiveDate,
    ) -> Result<DailyPriceSchedule> {
        let provider = &self.provider;
        self.cache
            .get_or_fetch(region, date, || async {
                provider.prices_for_date(region, date).await
            })
            .await
    }

    pub async fn recommend(&self, region: &Region) -> Result<String> {
        let schedule = self.current_price(region).await?;
        let now = Local::now().time();
        let current_hour = now.hour() as u8;

        let current = schedule.price_at_hour(current_hour);
        let cheapest = schedule.cheapest_period();
        let most_expensive = schedule.most_expensive_period();

        let mut recommendations = Vec::new();

        if let Some(period) = current {
            let tip = match period.tier {
                PriceTier::Valley => "当前是低谷时段，适合用电",
                PriceTier::Flat => "当前是平段时段，用电成本适中",
                PriceTier::Peak => "当前是高峰时段，建议减少用电",
            };
            recommendations.push(tip.to_string());
        }

        if let Some(cheap) = cheapest {
            recommendations.push(format!(
                "今日最低电价: {} {:02}:00-{:02}:00 ¥{:.4}/kWh",
                cheap.tier, cheap.start_hour, cheap.end_hour, cheap.price
            ));
        }

        if let Some(expensive) = most_expensive {
            recommendations.push(format!(
                "今日最高电价: {} {:02}:00-{:02}:00 ¥{:.4}/kWh",
                expensive.tier, expensive.start_hour, expensive.end_hour, expensive.price
            ));
        }

        if let (Some(curr), Some(cheap)) = (current, cheapest) {
            if curr.tier == PriceTier::Peak {
                let saving = curr.price - cheap.price;
                recommendations.push(format!(
                    "如果等到 {} {:02}:00-{:02}:00 用电，每 kWh 可省 ¥{:.4}",
                    cheap.tier, cheap.start_hour, cheap.end_hour, saving
                ));
            }
        }

        Ok(recommendations.join("\n"))
    }

    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }
}
