use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UsageType {
    Residential,
    Charging,
}

impl fmt::Display for UsageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UsageType::Residential => write!(f, "居民用电"),
            UsageType::Charging => write!(f, "充电桩"),
        }
    }
}

impl std::str::FromStr for UsageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "residential" | "居民" | "jumin" => Ok(UsageType::Residential),
            "charging" | "充电" | "chongdian" => Ok(UsageType::Charging),
            _ => Err(format!("未知的用电类型: {} (可选: residential/charging)", s)),
        }
    }
}