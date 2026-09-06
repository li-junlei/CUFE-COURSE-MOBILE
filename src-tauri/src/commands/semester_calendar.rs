// 学期开学时间表模块 - 内置表 + GitHub 静默刷新
//
// 数据文件 semester-calendar.json 位于仓库根：
//   顶层 key = 学年起始年（如 2025 表示 2025-2026 学年）
//   value   = { 学期(1/2) -> 第一周第一天(周一, "YYYY-MM-DD") }
// 维护规则：日期必须为周一且格式合法，否则该条目运行时被忽略；
// 新增学年后无需发版，App 会自动拉取远程文件。

use crate::models::{SemesterCalendar, SemesterCalendarCache};
use crate::storage::StorageManager;
use chrono::{Datelike, NaiveDate, Weekday};
use reqwest::Client;
use std::sync::OnceLock;
use std::time::Duration;

/// 远程时间表地址（GitHub raw 直连）
const RAW_URL: &str = "https://raw.githubusercontent.com/li-junlei/CUFE-COURSE-MOBILE/main/semester-calendar.json";
/// 备用地址（jsdelivr CDN，大陆可达性更好，有约 12~24h 缓存延迟）
const CDN_URL: &str = "https://cdn.jsdelivr.net/gh/li-junlei/CUFE-COURSE-MOBILE@main/semester-calendar.json";
/// 拉取间隔：60 天（仅成功拉取后推进，失败则下次启动重试）
const FETCH_INTERVAL_SECS: i64 = 60 * 24 * 60 * 60;
/// 内置时间表（随二进制打包，新装机/离线/拉取失败时兜底）
const BUILTIN: &str = include_str!("../../../semester-calendar.json");

/// 惰性解析内置时间表
fn builtin_calendar() -> &'static SemesterCalendar {
    static CACHE: OnceLock<SemesterCalendar> = OnceLock::new();
    CACHE.get_or_init(|| parse_calendar_text(BUILTIN).unwrap_or_default())
}

/// 校验日期为合法 YYYY-MM-DD 且为周一
fn is_valid_monday(date: &str) -> bool {
    matches!(
        NaiveDate::parse_from_str(date, "%Y-%m-%d"),
        Ok(d) if d.weekday() == Weekday::Mon
    )
}

/// 宽容解析时间表文本
/// - 整体非合法 JSON -> None
/// - 条目级非法（学年键非数字、学期非 1/2、日期非 YYYY-MM-DD 或非周一）-> 跳过该条目
/// - 全部条目被跳过 -> None（视为拉取失败，不覆盖旧缓存）
///
/// 先解析为 serde_json::Value 再手动提取：强类型 HashMap 反序列化遇到
/// "_comment" 等非嵌套 map 的键会整体失败，无法实现条目级宽容跳过。
fn parse_calendar_text(text: &str) -> Option<SemesterCalendar> {
    let raw: serde_json::Value = serde_json::from_str(text).ok()?;
    let root = raw.as_object()?;

    let mut calendar = SemesterCalendar::new();
    for (year_key, terms) in root {
        // 跳过 "_comment" 等非学年键
        let year: i32 = match year_key.parse() {
            Ok(y) => y,
            Err(_) => continue,
        };
        let Some(terms) = terms.as_object() else { continue };
        for (term_key, date) in terms {
            let Some(date) = date.as_str() else { continue };
            let term: i32 = match term_key.parse() {
                Ok(t) if t == 1 || t == 2 => t,
                _ => continue,
            };
            if !is_valid_monday(date) {
                continue;
            }
            calendar.entry(year).or_default().insert(term, date.to_string());
        }
    }

    if calendar.is_empty() {
        None
    } else {
        Some(calendar)
    }
}

/// 判断是否应发起拉取：距上次成功拉取超过间隔
/// 无记录 -> 拉取；时钟回拨（负值）-> 不拉取
fn should_fetch(last_fetched: Option<i64>, now: i64) -> bool {
    match last_fetched {
        None => true,
        Some(last) => now - last >= FETCH_INTERVAL_SECS,
    }
}

/// 在单张时间表中查询学年学期的开学日期
fn lookup(calendar: &SemesterCalendar, year: i32, term: i32) -> Option<String> {
    calendar.get(&year).and_then(|terms| terms.get(&term)).cloned()
}

/// 按优先级查询：远程缓存 -> 内置表
fn lookup_with_fallback(cached: Option<&SemesterCalendar>, year: i32, term: i32) -> Option<String> {
    if let Some(date) = cached.and_then(|c| lookup(c, year, term)) {
        return Some(date);
    }
    lookup(builtin_calendar(), year, term)
}

/// 静默拉取学期时间表（启动时调用，内部 60 天节流）
/// 任何失败不弹错、不写缓存、不推进时间戳，下次启动自然重试
#[tauri::command]
pub async fn fetch_semester_calendar() -> Result<(), String> {
    let storage = match StorageManager::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("创建存储管理器失败: {}", e);
            return Ok(());
        }
    };

    let now = chrono::Utc::now().timestamp();
    let last_fetched = storage.load_semester_calendar().map(|c| c.fetched_at);
    if !should_fetch(last_fetched, now) {
        return Ok(());
    }

    let client = match Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("CUFE-COURSE/1.0")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("创建 HTTP 客户端失败: {}", e);
            return Ok(());
        }
    };

    for url in [RAW_URL, CDN_URL] {
        let text = match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("读取学期时间表响应失败 {}: {}", url, e);
                    continue;
                }
            },
            Ok(resp) => {
                eprintln!("获取学期时间表失败 {}: {}", url, resp.status());
                continue;
            }
            Err(e) => {
                eprintln!("获取学期时间表失败 {}: {}", url, e);
                continue;
            }
        };

        match parse_calendar_text(&text) {
            Some(calendar) => {
                let cache = SemesterCalendarCache { fetched_at: now, calendar };
                if let Err(e) = storage.save_semester_calendar(&cache) {
                    eprintln!("保存学期时间表缓存失败: {}", e);
                }
                return Ok(());
            }
            None => {
                eprintln!("学期时间表内容无效: {}", url);
                continue;
            }
        }
    }

    Ok(())
}

/// 查询学年学期的开学日期：本地缓存 -> 内置表 -> None（前端回退启发式）
#[tauri::command]
pub fn get_semester_first_day(year: i32, term: i32) -> Option<String> {
    let cached = StorageManager::new()
        .ok()
        .and_then(|s| s.load_semester_calendar())
        .map(|c| c.calendar);
    lookup_with_fallback(cached.as_ref(), year, term)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_with_comment() {
        let text = r#"{
            "_comment": "维护说明",
            "2024": { "1": "2024-08-26", "2": "2025-02-17" },
            "2025": { "1": "2025-09-01" }
        }"#;
        let calendar = parse_calendar_text(text).unwrap();
        assert_eq!(lookup(&calendar, 2024, 1).unwrap(), "2024-08-26");
        assert_eq!(lookup(&calendar, 2024, 2).unwrap(), "2025-02-17");
        assert_eq!(lookup(&calendar, 2025, 1).unwrap(), "2025-09-01");
        assert_eq!(lookup(&calendar, 2025, 2), None);
        assert_eq!(lookup(&calendar, 2023, 1), None);
    }

    #[test]
    fn parse_skips_invalid_entries() {
        let text = r#"{
            "_comment": "说明",
            "abcd": { "1": "2024-08-26" },
            "2024": { "3": "2024-08-26" },
            "2025": { "1": "2025-02-30", "2": "2025-09-02" },
            "2026": { "1": "2026-02-16" }
        }"#;
        let calendar = parse_calendar_text(text).unwrap();
        // 非数字学年键、学期 3、非法日期 2-30、非周一 2025-09-02(周二) 全部跳过，仅 2026-1 存活
        assert_eq!(calendar.len(), 1);
        assert_eq!(lookup(&calendar, 2026, 1).unwrap(), "2026-02-16");
        assert_eq!(lookup(&calendar, 2024, 3), None);
        assert_eq!(lookup(&calendar, 2025, 1), None);
        assert_eq!(lookup(&calendar, 2025, 2), None);
    }

    #[test]
    fn parse_rejects_non_json() {
        assert!(parse_calendar_text("not json").is_none());
        assert!(parse_calendar_text("").is_none());
    }

    #[test]
    fn parse_rejects_all_invalid() {
        let text = r#"{
            "_comment": "只有说明",
            "2024": { "3": "2024-08-26" },
            "2025": { "1": "2025-09-02" }
        }"#;
        // 学期 3 非法、2025-09-02 非周一 -> 全部条目被跳过
        assert!(parse_calendar_text(text).is_none());
    }

    #[test]
    fn should_fetch_cases() {
        assert!(should_fetch(None, 1_000_000_000));
        assert!(!should_fetch(Some(1_000_000_000), 1_000_000_000 + FETCH_INTERVAL_SECS - 1));
        assert!(should_fetch(Some(1_000_000_000), 1_000_000_000 + FETCH_INTERVAL_SECS));
        // 时钟回拨（now 早于上次拉取）-> 不拉取
        assert!(!should_fetch(Some(1_000_000_000), 999_999_999));
    }

    #[test]
    fn lookup_prefers_cached_over_builtin() {
        let cached_text = r#"{ "2024": { "1": "2024-09-09" } }"#;
        let cached = parse_calendar_text(cached_text).unwrap();
        // 缓存命中优先
        assert_eq!(lookup_with_fallback(Some(&cached), 2024, 1).unwrap(), "2024-09-09");
        // 缓存缺失的学年回退内置表
        let from_builtin = lookup(builtin_calendar(), 2025, 1).unwrap();
        assert_eq!(lookup_with_fallback(Some(&cached), 2025, 1).unwrap(), from_builtin);
        // 无缓存时走内置表；两者都没有 -> None
        assert_eq!(lookup_with_fallback(None, 2025, 1).unwrap(), from_builtin);
        assert_eq!(lookup_with_fallback(None, 1999, 1), None);
    }

    /// 守护测试：内置时间表必须非空且全部条目合法（维护者写坏 JSON 时直接红）
    #[test]
    fn builtin_calendar_is_valid() {
        let calendar = builtin_calendar();
        assert!(!calendar.is_empty(), "内置时间表不能为空");
        for (year, terms) in calendar {
            for (term, date) in terms {
                assert!((1..=2).contains(term), "{}-{} 学期非法", year, term);
                assert!(is_valid_monday(&date), "{}-{} 日期 {} 非周一或格式非法", year, term, date);
            }
        }
    }
}
