use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveDate};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::error::{AppError, Result};
use crate::models::{DailyPriceSchedule, PricePeriod, PriceTier, Region, Season, UsageType};

pub struct SgccProvider {
    client: Client,
    config: AppConfig,
    usage_type: UsageType,
}

#[derive(Debug, Deserialize)]
struct SgccResponse {
    code: i32,
    msg: String,
    data: Option<Vec<SgccPriceItem>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SgccPriceItem {
    #[serde(rename = "timeType")]
    time_type: String,
    #[serde(rename = "startTime")]
    start_time: String,
    #[serde(rename = "endTime")]
    end_time: String,
    price: f64,
}

impl SgccProvider {
    pub fn new(config: AppConfig, usage_type: UsageType) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.api.timeout))
            .build()
            .map_err(|e| AppError::Api(format!("创建 HTTP 客户端失败: {}", e)))?;

        Ok(Self { client, config, usage_type })
    }

    fn build_api_url(&self, region: &Region, date: NaiveDate) -> String {
        format!(
            "{}/api/price/query?province={}&city={}&date={}",
            self.config.api.base_url, region.province, region.city, date
        )
    }

    async fn fetch_from_api(&self, region: &Region, date: NaiveDate) -> Result<Vec<PricePeriod>> {
        let url = self.build_api_url(region, date);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Api(format!("请求国网 API 失败: {}", e)))?;

        let body: SgccResponse = response
            .json()
            .await
            .map_err(|e| AppError::Api(format!("解析国网 API 响应失败: {}", e)))?;

        if body.code != 200 {
            return Err(AppError::Api(format!("API 错误: {}", body.msg)));
        }

        let items = body.data.ok_or_else(|| AppError::Api("API 返回数据为空".to_string()))?;

        let season = Season::from_month(date.month());
        let mut periods = Vec::new();

        for item in items {
            let tier = match item.time_type.as_str() {
                "peak" | "高" => PriceTier::Peak,
                "valley" | "低" => PriceTier::Valley,
                _ => PriceTier::Flat,
            };

            let start_hour = Self::parse_hour(&item.start_time)?;
            let end_hour = Self::parse_hour(&item.end_time)?;

            periods.push(PricePeriod {
                tier,
                start_hour,
                end_hour,
                price: item.price,
                season,
            });
        }

        Ok(periods)
    }

    fn parse_hour(time_str: &str) -> Result<u8> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if !parts.is_empty() {
            parts[0]
                .parse::<u8>()
                .map_err(|_| AppError::Parse(format!("解析时间失败: {}", time_str)))
        } else {
            Err(AppError::Parse(format!("无效的时间格式: {}", time_str)))
        }
    }

    fn build_fallback_schedule(&self, region: &Region, date: NaiveDate) -> DailyPriceSchedule {
        let season = Season::from_month(date.month());
        let pricing = match self.usage_type {
            UsageType::Residential => &self.config.pricing.residential,
            UsageType::Charging => &self.config.pricing.charging,
        };

        let mut periods = Vec::new();
        let schedule = &pricing.schedule;

        // Build a 24-hour timeline
        let mut hour_tiers: Vec<(u8, PriceTier, f64)> = Vec::new();

        // First, mark all hours as valley by default
        for hour in 0..24u8 {
            hour_tiers.push((hour, PriceTier::Valley, pricing.valley_price));
        }

        // Then mark peak hours
        for peak in &schedule.peak_hours {
            let start = peak[0];
            let end = peak[1];
            if start < end {
                for hour in start..end {
                    hour_tiers[hour as usize] = (hour, PriceTier::Peak, pricing.peak_price);
                }
            } else {
                // Wraps around midnight
                for hour in start..24 {
                    hour_tiers[hour as usize] = (hour, PriceTier::Peak, pricing.peak_price);
                }
                for hour in 0..end {
                    hour_tiers[hour as usize] = (hour, PriceTier::Peak, pricing.peak_price);
                }
            }
        }

        // Convert to periods by merging consecutive hours with same tier
        let mut current_tier = hour_tiers[0].1;
        let mut current_price = hour_tiers[0].2;
        let mut start_hour = 0u8;

        for hour in 1..24u8 {
            let (tier, price) = (hour_tiers[hour as usize].1, hour_tiers[hour as usize].2);
            if tier != current_tier || price != current_price {
                periods.push(PricePeriod {
                    tier: current_tier,
                    start_hour,
                    end_hour: hour,
                    price: current_price,
                    season,
                });
                current_tier = tier;
                current_price = price;
                start_hour = hour;
            }
        }
        // Add the last period
        periods.push(PricePeriod {
            tier: current_tier,
            start_hour,
            end_hour: 24,
            price: current_price,
            season,
        });

        DailyPriceSchedule {
            date,
            region: region.clone(),
            periods,
        }
    }
}

#[async_trait]
impl crate::api::traits::PriceProvider for SgccProvider {
    async fn current_price(&self, region: &Region) -> Result<DailyPriceSchedule> {
        self.today_prices(region).await
    }

    async fn today_prices(&self, region: &Region) -> Result<DailyPriceSchedule> {
        let today = Local::now().date_naive();
        self.prices_for_date(region, today).await
    }

    async fn prices_for_date(&self, region: &Region, date: NaiveDate) -> Result<DailyPriceSchedule> {
        match self.fetch_from_api(region, date).await {
            Ok(periods) => Ok(DailyPriceSchedule {
                date,
                region: region.clone(),
                periods,
            }),
            Err(e) => {
                tracing::warn!("API 请求失败，使用本地配置降级: {}", e);
                Ok(self.build_fallback_schedule(region, date))
            }
        }
    }

    fn name(&self) -> &str {
        "国家电网 (SGCC)"
    }

    fn supports_region(&self, region: &Region) -> bool {
        region.province == "jiangsu"
    }
}
