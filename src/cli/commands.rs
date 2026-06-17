use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};

use crate::config::AppConfig;
use crate::error::Result;
use crate::models::{Region, UsageType};
use crate::service::PriceService;

#[derive(Parser)]
#[command(name = "peak-valley-price")]
#[command(about = "峰谷电价查询工具 - 江苏无锡")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub province: Option<String>,

    #[arg(short, long, global = true)]
    pub city: Option<String>,

    #[arg(short, long, global = true, default_value = "residential")]
    pub r#type: UsageType,
}

#[derive(Subcommand)]
pub enum Commands {
    Current,
    Today,
    Date {
        #[arg(short, long)]
        date: NaiveDate,
    },
    Recommend,
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    Show,
    SetRegion {
        region: String,
    },
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load()?;

    let region = resolve_region(&cli, &config);
    let usage_type = cli.r#type;

    let provider = create_provider(&config, usage_type);
    let service = PriceService::new(provider, &config);

    match &cli.command {
        Commands::Current => {
            let schedule = service.current_price(&region).await?;
            let now = Local::now().time();

            println!("{} {} 当前电价:", usage_type, region);
            println!("{:-<40}", "");

            if let Some(period) = schedule.current_price(now) {
                println!("  当前时段: {} ({:02}:00-{:02}:00)",
                    period.tier, period.start_hour, period.end_hour);
                println!("  电价: ¥{:.4}/kWh", period.price);
            }

            println!("{:-<40}", "");
        }
        Commands::Today => {
            let schedule = service.today_prices(&region).await?;
            println!("{}", schedule);
        }
        Commands::Date { date } => {
            let schedule = service.prices_for_date(&region, *date).await?;
            println!("{}", schedule);
        }
        Commands::Recommend => {
            let recommendation = service.recommend(&region).await?;
            println!("用电建议:");
            println!("{:-<40}", "");
            println!("{}", recommendation);
            println!("{:-<40}", "");
        }
        Commands::Config { action } => {
            match action {
                ConfigAction::Show => {
                    println!("当前配置:");
                    println!("{:-<40}", "");
                    println!("  地区: {}", region);
                    println!("  用电类型: {}", usage_type);
                    println!("  API: {}", config.api.base_url);
                    println!("  缓存TTL: {}秒", config.cache.ttl);
                    match usage_type {
                        UsageType::Residential => {
                            println!("  高峰电价: ¥{:.4}/kWh", config.pricing.residential.peak_price);
                            println!("  低谷电价: ¥{:.4}/kWh", config.pricing.residential.valley_price);
                        }
                        UsageType::Charging => {
                            println!("  高峰电价: ¥{:.4}/kWh", config.pricing.charging.peak_price);
                            println!("  低谷电价: ¥{:.4}/kWh", config.pricing.charging.valley_price);
                        }
                    }
                    println!("{:-<40}", "");
                }
                ConfigAction::SetRegion { region: region_str } => {
                    let new_region: Region = region_str.parse()
                        .map_err(|e: String| crate::error::AppError::Config(e))?;
                    let mut new_config = config;
                    new_config.region.province = new_region.province.clone();
                    new_config.region.city = new_region.city.clone();
                    new_config.save()?;
                    println!("地区已设置为: {}", new_region);
                }
            }
        }
    }

    Ok(())
}

fn resolve_region(cli: &Cli, config: &AppConfig) -> Region {
    if let (Some(province), Some(city)) = (&cli.province, &cli.city) {
        Region::new(province, city)
    } else if let Some(province) = &cli.province {
        Region::new(province, &config.region.city)
    } else if let Some(city) = &cli.city {
        Region::new(&config.region.province, city)
    } else {
        Region::new(&config.region.province, &config.region.city)
    }
}

fn create_provider(config: &AppConfig, usage_type: UsageType) -> Box<dyn crate::api::traits::PriceProvider> {
    if let Ok(provider) = crate::api::SgccProvider::new(config.clone(), usage_type) {
        Box::new(provider)
    } else {
        tracing::warn!("无法创建国网 API 客户端，使用 Mock 数据");
        Box::new(crate::api::MockProvider::new(usage_type))
    }
}
