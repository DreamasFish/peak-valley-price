# peak-valley-price

峰谷电价查询工具 - 江苏无锡

一个用于查询中国峰谷电价的命令行工具，主要支持江苏无锡地区。

## 功能特性

- **实时电价查询**: 查看当前时段的电价信息
- **今日电价表**: 显示全天各时段电价
- **指定日期查询**: 查询任意日期的电价
- **智能用电建议**: 根据当前电价提供用电优化建议
- **多种用电类型**: 支持居民用电和充电桩用电
- **配置管理**: 支持自定义地区和配置参数
- **离线降级**: API 不可用时自动使用本地配置数据

## 安装

### 从源码编译

```bash
git clone https://github.com/DreamasFish/peak-valley-price.git
cd peak-valley-price
cargo build --release
```

编译后的可执行文件位于 `target/release/peak-valley-price`

## 使用方法

### 基本命令

```bash
# 查看今日电价（默认居民用电）
peak-valley-price today

# 查看充电桩电价
peak-valley-price -t charging today

# 查看当前电价
peak-valley-price current

# 查看充电桩当前电价
peak-valley-price -t charging current

# 查询指定日期电价
peak-valley-price date --date 2024-01-15

# 获取用电建议
peak-valley-price recommend
```

### 用电类型

```bash
# 居民用电（默认）
peak-valley-price -t residential today

# 充电桩用电
peak-valley-price -t charging today
```

### 配置管理

```bash
# 查看当前配置
peak-valley-price config show

# 设置地区（格式：省份/城市）
peak-valley-price config set-region jiangsu/wuxi
```

### 地区参数

```bash
# 通过命令行参数指定地区
peak-valley-price -p jiangsu -c wuxi today
```

## 电价时段说明

### 居民用电

| 时段 | 时间 | 电价 (元/kWh) |
|------|------|---------------|
| 高峰 | 08:00-21:00 | 0.5583 |
| 低谷 | 21:00-08:00 | 0.3583 |

### 充电桩用电

| 时段 | 时间 | 电价 (元/kWh) |
|------|------|---------------|
| 高峰 | 07:00-11:00, 13:00-22:00 | 0.5783 |
| 低谷 | 11:00-13:00, 22:00-07:00 | 0.3783 |

## 支持的地区

| 省份 | 城市 |
|------|------|
| 江苏 | 无锡、南京、苏州 |
| 浙江 | 杭州 |
| 上海 | 上海 |
| 广东 | 深圳 |
| 北京 | 北京 |

## 配置文件

配置文件位于系统配置目录：
- Linux/macOS: `~/.config/peak-valley-price/config.toml`
- Windows: `%APPDATA%\peak-valley-price\config.toml`

### 默认配置示例

```toml
[region]
province = "jiangsu"
city = "wuxi"

[api]
base_url = "https://www.95598.cn"
timeout = 10

[cache]
ttl = 3600

# 居民用电
[pricing.residential]
peak_price = 0.5583
valley_price = 0.3583

[pricing.residential.schedule]
peak_hours = [[8, 21]]
valley_hours = [[21, 8]]

# 充电桩用电
[pricing.charging]
peak_price = 0.5783
valley_price = 0.3783

[pricing.charging.schedule]
peak_hours = [[7, 11], [13, 22]]
valley_hours = [[11, 13], [22, 7]]
```

## 项目结构

```
src/
├── main.rs          # 程序入口
├── lib.rs           # 库根模块
├── cli/             # 命令行接口
│   ├── mod.rs
│   └── commands.rs  # 命令定义和处理
├── api/             # API 客户端
│   ├── traits.rs    # Provider trait 定义
│   ├── sgcc.rs      # 国家电网 API 实现
│   └── mock.rs      # 测试数据提供者
├── models/          # 数据模型
│   ├── price.rs     # 电价相关模型
│   ├── price_type.rs # 用电类型
│   └── region.rs    # 地区模型
├── service/         # 业务逻辑
│   ├── price_service.rs  # 电价服务
│   └── cache.rs     # 缓存实现
├── config.rs        # 配置管理
└── error.rs         # 错误处理
```

## 依赖

- `tokio` - 异步运行时
- `reqwest` - HTTP 客户端
- `clap` - 命令行参数解析
- `chrono` - 日期时间处理
- `serde` / `serde_json` - 序列化
- `tracing` - 日志追踪

## 开发

### 运行测试

```bash
cargo test
```

### 代码检查

```bash
cargo clippy
cargo fmt --check
```

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！