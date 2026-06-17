use async_trait::async_trait;
use chrono::NaiveDate;

use crate::error::Result;
use crate::models::{DailyPriceSchedule, Region};

#[async_trait]
pub trait PriceProvider: Send + Sync {
    async fn current_price(&self, region: &Region) -> Result<DailyPriceSchedule>;
    async fn today_prices(&self, region: &Region) -> Result<DailyPriceSchedule>;
    async fn prices_for_date(&self, region: &Region, date: NaiveDate) -> Result<DailyPriceSchedule>;
    fn name(&self) -> &str;
    fn supports_region(&self, region: &Region) -> bool;
}
