use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveDate};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::error::{AppError, Result};
use crate::models::{DailyPriceSchedule, PricePeriod, PriceTier, Region, Season};

pub struct SgccProvider {
    client: Client,
    config: AppConfig,
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
    pub fn new(config: AppConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.api.timeout))
            .build()
            .map_err(|e| AppError::Api(format!("创建 HTTP 客户端失败: {}", e)))?;

        Ok(Self { client, config })
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
        let config = &self.config.pricing;

        let mut periods = Vec::new();
        let mut current_hour = 0u8;

        let schedule = &config.schedule;
        let default_price = &config.default;

        for peak in &schedule.peak_hours {
            if peak[0] != current_hour {
                periods.push(PricePeriod {
                    tier: PriceTier::Flat,
                    start_hour: current_hour,
                    end_hour: peak[0],
                    price: default_price.flat_price,
                    season,
                });
            }
            periods.push(PricePeriod {
                tier: PriceTier::Peak,
                start_hour: peak[0],
                end_hour: peak[1],
                price: default_price.peak_price,
                season,
            });
            current_hour = peak[1];
        }

        for valley in &schedule.valley_hours {
            if valley[0] != current_hour {
                periods.push(PricePeriod {
                    tier: PriceTier::Flat,
                    start_hour: current_hour,
                    end_hour: valley[0],
                    price: default_price.flat_price,
                    season,
                });
            }
            periods.push(PricePeriod {
                tier: PriceTier::Valley,
                start_hour: valley[0],
                end_hour: valley[1],
                price: default_price.valley_price,
                season,
            });
            current_hour = valley[1];
        }

        if current_hour < 24 {
            periods.push(PricePeriod {
                tier: PriceTier::Flat,
                start_hour: current_hour,
                end_hour: 24,
                price: default_price.flat_price,
                season,
            });
        }

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
