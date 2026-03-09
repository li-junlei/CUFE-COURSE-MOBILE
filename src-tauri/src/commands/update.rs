// 更新检查模块 - GitHub Release 检查逻辑

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// GitHub Release 响应结构
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    html_url: String,
}

/// 返回给前端的更新信息
#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub title: String,
    #[serde(rename = "releaseNotes")]
    pub release_notes: String,
    #[serde(rename = "releaseUrl")]
    pub release_url: String,
}

/// 比较版本号
/// 返回: 1=latest > current, 0=相等, -1=current > latest
fn compare_versions(current: &str, latest: &str) -> i32 {
    let parse_version = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let current_parts = parse_version(current);
    let latest_parts = parse_version(latest);

    for (c, l) in current_parts.iter().zip(latest_parts.iter()) {
        if l > c {
            return 1;
        }
        if c > l {
            return -1;
        }
    }

    // 长度比较：2.4.0 vs 2.4.0.1 -> 2.4.0.1 更新
    if latest_parts.len() > current_parts.len() {
        return 1;
    }
    if current_parts.len() > latest_parts.len() {
        return -1;
    }

    0
}

/// 检查更新命令
///
/// # 参数
/// - `current_version`: 当前应用版本号 (如 "2.4.0")
/// - `auto_check`: 是否为自动检查 (自动检查时跳过已忽略版本)
/// - `skipped_version`: 用户跳过的版本号
///
/// # 返回
/// - `Some(UpdateInfo)`: 发现新版本
/// - `None`: 无新版本或需要跳过
#[tauri::command]
pub async fn check_update(
    current_version: String,
    auto_check: bool,
    skipped_version: Option<String>,
) -> Result<Option<UpdateInfo>, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("CUFE-COURSE/1.0")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = "https://api.github.com/repos/li-junlei/CUFE-COURSE/releases/latest";

    let response = match client.get(url).header("Accept", "application/vnd.github+json").send().await {
        Ok(resp) => resp,
        Err(e) => {
            // 网络错误：自动检查静默处理，手动检查返回错误
            if auto_check {
                eprintln!("自动检查更新失败: {}", e);
                return Ok(None);
            }
            return Err(format!("网络连接失败: {}", e));
        }
    };

    if !response.status().is_success() {
        if auto_check {
            eprintln!("GitHub API 返回错误: {}", response.status());
            return Ok(None);
        }
        return Err(format!("GitHub API 返回错误: {}", response.status()));
    }

    let release: GitHubRelease = match response.json().await {
        Ok(r) => r,
        Err(e) => {
            if auto_check {
                eprintln!("解析响应失败: {}", e);
                return Ok(None);
            }
            return Err(format!("解析响应失败: {}", e));
        }
    };

    let latest_version = release.tag_name.trim_start_matches('v');

    // 比较版本
    let cmp = compare_versions(&current_version, latest_version);
    if cmp <= 0 {
        // 当前版本 >= 最新版本，无需更新
        return Ok(None);
    }

    // 自动检查时：检查是否跳过此版本
    if auto_check {
        if let Some(ref skipped) = skipped_version {
            let skip_cmp = compare_versions(skipped, latest_version);
            if skip_cmp == 0 {
                // 跳过此版本
                return Ok(None);
            }
        }
    }

    // 返回更新信息
    Ok(Some(UpdateInfo {
        version: latest_version.to_string(),
        title: release.name.unwrap_or_else(|| release.tag_name),
        release_notes: release.body.unwrap_or_default(),
        release_url: release.html_url,
    }))
}
