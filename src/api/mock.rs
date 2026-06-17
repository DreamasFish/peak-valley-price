use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveDate};

use crate::error::Result;
use crate::models::{DailyPriceSchedule, PricePeriod, PriceTier, Region, Season, UsageType};

pub struct MockProvider {
    usage_type: UsageType,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new(UsageType::Residential)
    }
}

impl MockProvider {
    pub fn new(usage_type: UsageType) -> Self {
        Self { usage_type }
    }

    fn build_mock_schedule(&self, region: &Region, date: NaiveDate) -> DailyPriceSchedule {
        let season = Season::from_month(date.month());

        let periods = match self.usage_type {
            UsageType::Residential => vec![
                PricePeriod {
                    tier: PriceTier::Valley,
                    start_hour: 0,
                    end_hour: 8,
                    price: 0.3583,
                    season,
                },
                PricePeriod {
                    tier: PriceTier::Peak,
                    start_hour: 8,
                    end_hour: 21,
                    price: 0.5583,
                    season,
                },
                PricePeriod {
                    tier: PriceTier::Valley,
                    start_hour: 21,
                    end_hour: 24,
                    price: 0.3583,
                    season,
                },
            ],
            UsageType::Charging => vec![
                PricePeriod {
                    tier: PriceTier::Valley,
                    start_hour: 0,
                    end_hour: 7,
                    price: 0.3783,
                    season,
                },
                PricePeriod {
                    tier: PriceTier::Peak,
                    start_hour: 7,
                    end_hour: 11,
                    price: 0.5783,
                    season,
                },
                PricePeriod {
                    tier: PriceTier::Valley,
                    start_hour: 11,
                    end_hour: 13,
                    price: 0.3783,
                    season,
                },
                PricePeriod {
                    tier: PriceTier::Peak,
                    start_hour: 13,
                    end_hour: 22,
                    price: 0.5783,
                    season,
                },
                PricePeriod {
                    tier: PriceTier::Valley,
                    start_hour: 22,
                    end_hour: 24,
                    price: 0.3783,
                    season,
                },
            ],
        };

        DailyPriceSchedule {
            date,
            region: region.clone(),
            periods,
        }
    }
}

#[async_trait]
impl crate::api::traits::PriceProvider for MockProvider {
    async fn current_price(&self, region: &Region) -> Result<DailyPriceSchedule> {
        self.today_prices(region).await
    }

    async fn today_prices(&self, region: &Region) -> Result<DailyPriceSchedule> {
        let today = Local::now().date_naive();
        self.prices_for_date(region, today).await
    }

    async fn prices_for_date(&self, region: &Region, date: NaiveDate) -> Result<DailyPriceSchedule> {
        Ok(self.build_mock_schedule(region, date))
    }

    fn name(&self) -> &str {
        "Mock (测试数据)"
    }

    fn supports_region(&self, _region: &Region) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::traits::PriceProvider;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider::new(UsageType::Residential);
        let region = Region::jiangsu_wuxi();
        let schedule = provider.today_prices(&region).await.unwrap();

        assert!(!schedule.periods.is_empty());
        assert!(schedule.periods.iter().any(|p| p.tier == PriceTier::Peak));
        assert!(schedule.periods.iter().any(|p| p.tier == PriceTier::Valley));
    }
}
