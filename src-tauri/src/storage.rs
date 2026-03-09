use crate::models::{AppConfig, CachedSchedule, ScheduleMetadata};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid; // 需要添加 uuid 依赖

/// 存储管理器
pub struct StorageManager {
    data_dir: PathBuf,
}

impl StorageManager {
    /// 创建存储管理器
    pub fn new() -> Result<Self, String> {
        // 获取用户数据目录（兼容 Android）
        let data_dir = dirs::data_local_dir()
            .or_else(|| dirs::data_dir())
            .or_else(|| {
                // Android fallback: 使用内部存储目录
                let android_path = std::path::PathBuf::from("/data/data/com.lijunlei.cufecourse/files");
                if android_path.exists() || std::fs::create_dir_all(&android_path).is_ok() {
                    Some(android_path)
                } else {
                    None
                }
            })
            .ok_or("无法获取用户数据目录")?
            .join("cufe-course");

        // 创建目录（如果不存在）
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("创建数据目录失败: {}", e))?;

        // 创建课表目录
        let schedules_dir = data_dir.join("schedules");
        fs::create_dir_all(&schedules_dir)
            .map_err(|e| format!("创建课表目录失败: {}", e))?;

        Ok(StorageManager { data_dir })
    }

    /// 获取配置文件路径
    fn config_path(&self) -> PathBuf {
        self.data_dir.join("config.json")
    }

    /// 获取课表存储目录
    fn schedules_dir(&self) -> PathBuf {
        self.data_dir.join("schedules")
    }

    /// 获取指定ID的课表文件路径
    fn schedule_path(&self, id: &str) -> PathBuf {
        self.schedules_dir().join(format!("{}.json", id))
    }

    /// 获取 Cookie 文件路径
    pub fn cookie_path(&self) -> PathBuf {
        self.data_dir.join("cookie.txt")
    }

    /// 获取背景图目录
    pub fn background_dir(&self) -> PathBuf {
        let dir = self.data_dir.join("backgrounds");
        fs::create_dir_all(&dir).ok();
        dir
    }

    /// 加载应用配置
    pub fn load_config(&self) -> Result<AppConfig, String> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(AppConfig::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        match serde_json::from_str(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                println!("解析配置文件失败, 重置为默认配置: {}", e);
                // 备份损坏的配置文件
                let backup_path = path.with_extension("json.bak");
                let _ = fs::rename(&path, &backup_path);
                Ok(AppConfig::default())
            }
        }
    }

    /// 保存应用配置
    pub fn save_config(&self, config: &AppConfig) -> Result<(), String> {
        let path = self.config_path();
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;

        let mut file = fs::File::create(&path)
            .map_err(|e| format!("创建配置文件失败: {}", e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("写入配置文件失败: {}", e))?;

        Ok(())
    }

    /// 加载指定ID的课表数据
    pub fn load_schedule(&self, id: &str) -> Result<CachedSchedule, String> {
        let path = self.schedule_path(id);
        if !path.exists() {
            return Err(format!("课表 {} 不存在", id));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("读取课表缓存失败: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("解析课表缓存失败: {}", e))
    }

    /// 保存课表缓存（自动生成ID）
    pub fn save_schedule(&self, cached: &CachedSchedule) -> Result<(), String> {
        let path = self.schedule_path(&cached.id);
        let content = serde_json::to_string_pretty(cached)
            .map_err(|e| format!("序列化课表数据失败: {}", e))?;

        let mut file = fs::File::create(&path)
            .map_err(|e| format!("创建课表缓存文件失败: {}", e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("写入课表缓存失败: {}", e))?;

        Ok(())
    }

    /// 获取所有课表的元数据列表
    pub fn list_schedules(&self) -> Result<Vec<ScheduleMetadata>, String> {
        let schedules_dir = self.schedules_dir();
        if !schedules_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&schedules_dir)
            .map_err(|e| format!("读取课表目录失败: {}", e))?;

        let mut schedules = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            // 只处理 .json 文件
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // 读取文件内容
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("读取课表文件失败: {}", e))?;

            // 解析为 CachedSchedule
            if let Ok(cached) = serde_json::from_str::<CachedSchedule>(&content) {
                let metadata = ScheduleMetadata {
                    id: cached.id.clone(),
                    name: cached.name.clone(),
                    created_at: cached.timestamp,
                    updated_at: cached.timestamp,
                    // 课程数统计排除考试
                    course_count: cached.courses.iter()
                        .filter(|c| c.course_type != crate::models::CourseType::Exam)
                        .count(),
                    first_day: cached.first_day,
                    max_periods: cached.max_periods,
                    weeks_count: cached.weeks_count,
                    time_table_id: cached.time_table_id.clone(),
                    sort_index: cached.sort_index,
                    school_year: cached.school_year,
                    school_term: cached.school_term,
                };
                schedules.push(metadata);
            }
        }

        // 排序规则: sort_index (升序) -> updated_at (降序)
        schedules.sort_by(|a, b| {
            match (a.sort_index, b.sort_index) {
                (Some(idx_a), Some(idx_b)) => {
                    if idx_a != idx_b {
                        return idx_a.cmp(&idx_b);
                    }
                    // 索引相同，按时间倒序
                    b.updated_at.cmp(&a.updated_at)
                },
                (Some(_), None) => std::cmp::Ordering::Less, // 有索引的排前面
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => b.updated_at.cmp(&a.updated_at), // 都没索引，按时间倒序
            }
        });

        Ok(schedules)
    }

    /// 删除指定ID的课表
    pub fn delete_schedule(&self, id: &str) -> Result<(), String> {
        let path = self.schedule_path(id);
        if !path.exists() {
            return Err(format!("课表 {} 不存在", id));
        }

        fs::remove_file(&path)
            .map_err(|e| format!("删除课表文件失败: {}", e))?;

        Ok(())
    }

    /// 生成新的课表ID
    pub fn generate_schedule_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// ============================================================
    /// 凭证管理方法（持久化登录）
    /// ============================================================

    /// 获取凭证文件路径
    fn credentials_path(&self) -> std::path::PathBuf {
        self.data_dir.join("credentials.json")
    }

    /// 保存加密凭证
    pub fn save_credentials(&self, credentials: &crate::models::PersistentCredentials) -> Result<(), String> {
        let path = self.credentials_path();
        let content = serde_json::to_string_pretty(credentials)
            .map_err(|e| format!("序列化凭证失败: {}", e))?;

        fs::File::create(&path)
            .and_then(|mut file| file.write_all(content.as_bytes()))
            .map_err(|e| format!("创建凭证文件失败 (路径: {}): {}",
                                path.display(), e))
    }

    /// 加载加密凭证
    pub fn load_credentials(&self) -> Result<crate::models::PersistentCredentials, String> {
        let path = self.credentials_path();
        if !path.exists() {
            return Err("未找到保存的凭证".to_string());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("读取凭证文件失败: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("解析凭证文件失败: {}", e))
    }

    /// 清除所有登录凭证
    pub fn clear_credentials(&self) -> Result<(), String> {
        let path = self.credentials_path();
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("删除凭证文件失败: {}", e))?;
        }
        Ok(())
    }

    /// 删除背景图
    pub fn delete_background(&self, filename: &str) -> Result<(), String> {
        let path = self.background_dir().join(filename);
        if !path.exists() {
            return Ok(());
        }

        fs::remove_file(&path)
            .map_err(|e| format!("删除背景图失败: {}", e))?;

        Ok(())
    }

    /// 清空所有数据
    pub fn clear_all(&self) -> Result<(), String> {
        let config_path = self.config_path();
        let schedules_dir = self.schedules_dir();
        let cookie_path = self.cookie_path();
        let username_path = self.data_dir.join("username.txt");

        if config_path.exists() {
            fs::remove_file(&config_path).ok();
        }
        // 删除整个课表目录
        if schedules_dir.exists() {
            fs::remove_dir_all(&schedules_dir).map_err(|e| format!("删除课表目录失败: {}", e))?;
            // 重新创建目录
            fs::create_dir_all(&schedules_dir).map_err(|e| format!("创建课表目录失败: {}", e))?;
        }
        if cookie_path.exists() {
            fs::remove_file(&cookie_path).ok();
        }
        if username_path.exists() {
            fs::remove_file(&username_path).ok();
        }

        Ok(())
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new().expect("Failed to create StorageManager")
    }
}
