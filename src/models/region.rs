use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub province: String,
    pub city: String,
}

impl Region {
    pub fn new(province: &str, city: &str) -> Self {
        Self {
            province: province.to_lowercase(),
            city: city.to_lowercase(),
        }
    }

    pub fn jiangsu_wuxi() -> Self {
        Self::new("jiangsu", "wuxi")
    }

    pub fn display_name(&self) -> String {
        let province = match self.province.as_str() {
            "jiangsu" => "江苏",
            "zhejiang" => "浙江",
            "shanghai" => "上海",
            "guangdong" => "广东",
            "beijing" => "北京",
            _ => &self.province,
        };
        let city = match self.city.as_str() {
            "wuxi" => "无锡",
            "nanjing" => "南京",
            "suzhou" => "苏州",
            "hangzhou" => "杭州",
            "shenzhen" => "深圳",
            _ => &self.city,
        };
        format!("{} {}", province, city)
    }

    pub fn supported_regions() -> Vec<Region> {
        vec![
            Region::jiangsu_wuxi(),
            Region::new("jiangsu", "nanjing"),
            Region::new("jiangsu", "suzhou"),
            Region::new("zhejiang", "hangzhou"),
            Region::new("shanghai", "shanghai"),
            Region::new("guangdong", "shenzhen"),
            Region::new("beijing", "beijing"),
        ]
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for Region {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 2 {
            Ok(Region::new(parts[0], parts[1]))
        } else {
            Err(format!("地区格式错误: {} (应为 province/city)", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_creation() {
        let region = Region::new("jiangsu", "wuxi");
        assert_eq!(region.province, "jiangsu");
        assert_eq!(region.city, "wuxi");
    }

    #[test]
    fn test_region_display() {
        let region = Region::jiangsu_wuxi();
        assert_eq!(region.display_name(), "江苏 无锡");
    }

    #[test]
    fn test_region_from_str() {
        let region: Region = "jiangsu/wuxi".parse().unwrap();
        assert_eq!(region, Region::jiangsu_wuxi());
    }
}
