use chrono::{NaiveDate, Timelike};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::models::region::Region;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceTier {
    Peak,
    Flat,
    Valley,
}

impl fmt::Display for PriceTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriceTier::Peak => write!(f, "高峰"),
            PriceTier::Flat => write!(f, "平段"),
            PriceTier::Valley => write!(f, "低谷"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Summer,
    Winter,
    Spring,
    Autumn,
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Season::Summer => write!(f, "夏季"),
            Season::Winter => write!(f, "冬季"),
            Season::Spring => write!(f, "春季"),
            Season::Autumn => write!(f, "秋季"),
        }
    }
}

impl Season {
    pub fn from_month(month: u32) -> Self {
        match month {
            7 | 8 => Season::Summer,
            12 | 1 => Season::Winter,
            3..=5 => Season::Spring,
            _ => Season::Autumn,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePeriod {
    pub tier: PriceTier,
    pub start_hour: u8,
    pub end_hour: u8,
    pub price: f64,
    pub season: Season,
}

impl PricePeriod {
    pub fn contains_hour(&self, hour: u8) -> bool {
        if self.start_hour < self.end_hour {
            hour >= self.start_hour && hour < self.end_hour
        } else {
            hour >= self.start_hour || hour < self.end_hour
        }
    }

    pub fn duration_hours(&self) -> u8 {
        if self.start_hour < self.end_hour {
            self.end_hour - self.start_hour
        } else {
            (24 - self.start_hour) + self.end_hour
        }
    }
}

impl fmt::Display for PricePeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:02}:00-{:02}:00 ¥{:.2}/kWh",
            self.tier, self.start_hour, self.end_hour, self.price
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPriceSchedule {
    pub date: NaiveDate,
    pub region: Region,
    pub periods: Vec<PricePeriod>,
}

impl DailyPriceSchedule {
    pub fn price_at_hour(&self, hour: u8) -> Option<&PricePeriod> {
        self.periods.iter().find(|p| p.contains_hour(hour))
    }

    pub fn current_price(&self, now: chrono::NaiveTime) -> Option<&PricePeriod> {
        self.price_at_hour(now.hour() as u8)
    }

    pub fn cheapest_period(&self) -> Option<&PricePeriod> {
        self.periods
            .iter()
            .min_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn most_expensive_period(&self) -> Option<&PricePeriod> {
        self.periods
            .iter()
            .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal))
    }
}

impl fmt::Display for DailyPriceSchedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "📅 {} {} 电价表:", self.date, self.region)?;
        writeln!(f, "{:-<40}", "")?;
        for period in &self.periods {
            writeln!(f, "  {}", period)?;
        }
        writeln!(f, "{:-<40}", "")?;
        if let Some(cheap) = self.cheapest_period() {
            writeln!(f, "💡 最便宜: {} {:02}:00-{:02}:00 ¥{:.4}/kWh",
                cheap.tier, cheap.start_hour, cheap.end_hour, cheap.price)?;
        }
        Ok(())
    }
}
