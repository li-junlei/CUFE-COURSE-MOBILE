use crate::models::{Course, LoginResponse, UserCredentials};
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Duration;
use std::sync::{Arc, Mutex};

/// 教务系统客户端
pub struct EduSystemClient {
    client: Client,
    base_url: String,
    #[allow(dead_code)]
    cookie: Option<String>,  // 保留以兼容旧代码，但不再使用
    username: Option<String>,  // 学号，用于查询课表时的 su 参数
}

/// 全局教务系统状态（用于 Tauri 状态管理）
pub struct EduSystemState {
    client: Arc<Mutex<Option<EduSystemClient>>>,
    base_url: String,
}

impl EduSystemState {
    /// 创建新的状态实例
    pub fn new(base_url: String) -> Self {
        EduSystemState {
            client: Arc::new(Mutex::new(None)),
            base_url,
        }
    }

    /// 初始化客户端（登录时调用）
    pub fn initialize_client(&self, username: String) {
        let mut client_guard = self.client.lock().unwrap();
        *client_guard = Some(EduSystemClient::new(self.base_url.clone(), Some(username)));
    }

    /// 获取客户端引用
    pub fn get_client(&self) -> Result<EduSystemClient, String> {
        let client_guard = self.client.lock().unwrap();
        client_guard.as_ref()
            .map(|c| EduSystemClient {
                client: c.client.clone(),
                base_url: c.base_url.clone(),
                cookie: c.cookie.clone(),
                username: c.username.clone(),
            })
            .ok_or_else(|| "客户端未初始化，请先登录".to_string())
    }

    /// 检查是否已登录
    pub fn is_logged_in(&self) -> bool {
        let client_guard = self.client.lock().unwrap();
        client_guard.is_some()
    }

    /// 登出（清除客户端）
    pub fn logout(&self) {
        let mut client_guard = self.client.lock().unwrap();
        *client_guard = None;
    }

    /// ============================================================
    /// 会话管理方法（持久化登录）
    /// ============================================================

    /// 自动重新登录（使用保存的凭证）
    pub async fn auto_relogin(&self, credentials: &crate::models::UserCredentials) -> Result<(), String> {
        // 重新初始化客户端
        self.initialize_client(credentials.username.clone());

        // 获取客户端并执行登录
        let mut client = self.get_client()?;
        let login_response = client.login(credentials).await?;

        if !login_response.success {
            self.logout();
            return Err(format!("自动登录失败: {}", login_response.message));
        }

        Ok(())
    }
}

// 为其他线程安全地使用状态实现 Clone
impl Clone for EduSystemClient {
    fn clone(&self) -> Self {
        EduSystemClient {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            cookie: None,  // 不再需要克隆 cookie
            username: self.username.clone(),
        }
    }
}

impl EduSystemClient {
    /// 创建新的客户端
    pub fn new(base_url: String, username: Option<String>) -> Self {
        // 创建一个带有 Cookie 存储的客户端
        // reqwest 的 cookie_store 会自动管理会话 cookie
        let client = Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(30))
            // 模拟浏览器 User-Agent
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");

        EduSystemClient {
            client,
            base_url: base_url.trim_end_matches('/').to_string(), // 移除末尾斜杠
            cookie: None,  // 不再手动管理 cookie
            username,
        }
    }

    /// 获取当前时间戳（毫秒）
    fn get_timestamp() -> u128 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis()
    }

    /// 登录页当前仍走 HTTPS，但登录后的业务接口实际跑在 HTTP。
    fn api_base_url(&self) -> String {
        if self.base_url.starts_with("https://") {
            format!("http://{}", &self.base_url["https://".len()..])
        } else {
            self.base_url.clone()
        }
    }

    /// 执行登录
    /// 参考 C:\project\new-school-sdk\test_cufe.py 的实现
    /// 使用 plaintext 登录 bypass (mmsfjm: '0')
    pub async fn login(
        &mut self,
        credentials: &UserCredentials,
    ) -> Result<LoginResponse, String> {
        let timestamp = Self::get_timestamp();
        let login_url = format!("{}/xtgl/login_slogin.html?time={}", self.base_url, timestamp);

        // 1. GET 请求获取 CSRF Token
        let response = self.client.get(&login_url)
            .send()
            .await
            .map_err(|e| format!("无法访问登录页面: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("无法访问登录页面: HTTP {}", response.status()));
        }

        // 打印初始请求的响应头（可能包含 Cookie）
        let mut cookies_str = String::new();
        for (key, value) in response.headers() {
            // 提取 Cookie（GET 请求时就会设置）
            if key.as_str().eq_ignore_ascii_case("set-cookie") {
                if let Ok(v) = value.to_str() {
                    if !cookies_str.is_empty() {
                        cookies_str.push_str("; ");
                    }
                    // 提取 cookie 的 key=value 部分
                    let part = v.split(';').next().unwrap_or(v);
                    cookies_str.push_str(part);
                }
            }
        }

        // 保存从 GET 请求获取的 Cookie
        if !cookies_str.is_empty() {
            self.cookie = Some(cookies_str.clone());
        }

        let html_content = response.text().await.map_err(|e| format!("读取登录页面失败: {}", e))?;
        
        // 解析 CSRF Token
        let csrftoken = {
            let document = Html::parse_document(&html_content);
            let params_selector = Selector::parse("input#csrftoken").unwrap();
            document.select(&params_selector).next()
                .and_then(|el| el.value().attr("value"))
                .ok_or("无法获取 CSRF Token")?
                .to_string()
        };


        // 2. 构造登录参数
        // 参考 test_cufe.py: mmsfjm='0' 强制明文传输
        let params = [
            ("csrftoken", csrftoken.as_str()),
            ("yhm", &credentials.username),
            ("mm", &credentials.password),
            ("mmsfjm", "0"),
        ];

        // 3. 发送登录请求
        let login_submit_url = format!("{}/xtgl/login_slogin.html?time={}", self.base_url, Self::get_timestamp());
        
        let response = self.client.post(&login_submit_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("登录请求失败: {}", e))?;

        if !response.status().is_success() {
             return Err(format!("登录请求返回错误: HTTP {}", response.status()));
        }

        // 打印登录响应头（调试用）

        let response_text = response.text().await.map_err(|e| format!("读取登录响应失败: {}", e))?;

        // 4. 验证登录是否成功
        // 检查返回内容中是否包含用户名（参考 SDK 的 _is_login）
        if response_text.contains(&format!("value=\"{}\"", credentials.username)) || response_text.contains("xsxx_update.html") {
            Ok(LoginResponse {
                success: true,
                message: "登录成功".to_string(),
                cookie: self.cookie.clone(),
            })
        } else {
            // 尝试解析错误信息
            let doc = Html::parse_document(&response_text);
            let tips_selector = Selector::parse("#tips").unwrap();
            let error_msg = doc.select(&tips_selector).next()
                .map(|el| el.text().collect::<Vec<_>>().join(""))
                .unwrap_or_else(|| "登录失败，可能是用户名或密码错误".to_string());
            
            Err(error_msg)
        }
    }

    /// 获取课表数据 (CUFE)
    /// 自动获取当前学期的课表
    pub async fn get_schedule(&self, year: i32, term: i32) -> Result<Vec<Course>, String> {
        // 参考 SDK schedules.py：需要 su 参数（学号）
        let username = self.username.as_ref()
            .ok_or("获取课表需要学号信息，请重新登录".to_string())?;

        let url = format!("{}/kbcx/xskbcx_cxXsKb.html", self.api_base_url());

        // 构造请求参数，参考 SDK schedules.py
        // xqm: 学期码 (3: 第一学期, 12: 第二学期, 16: 第三学期) - 根据 SDK 的 TERM 字典推断
        // SDK 中 TERM = {1: 3, 2: 12, 3: 16}
        let term_code = match term {
            1 => "3",
            2 => "12",
            3 => "16",
            _ => "3", // 默认为第一学期
        };

        // 查询参数（参考 Python SDK）
        let query_params = [
            ("gnmkdm", "N2151"),
            ("su", username),
        ];

        // POST 数据（表单数据）
        let form_params = [
            ("xnm", year.to_string()),
            ("xqm", term_code.to_string()),
            ("kzlx", "ck".to_string()),
        ];


        // 构建请求 - 如果有手动设置的cookie，需要添加到请求头
        let mut request = self.client.post(&url)
            .query(&query_params)  // 添加查询参数 gnmkdm 和 su
            .form(&form_params)    // 添加表单数据 xnm, xqm, kzlx
            .header("Content-Type", "application/x-www-form-urlencoded");

        // 如果有手动设置的Cookie（从文件恢复的），手动添加到请求头
        if let Some(ref cookie) = self.cookie {
            request = request.header("Cookie", cookie);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("获取课表请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("获取课表失败: HTTP {}", response.status()));
        }

        let json_text = response.text().await.map_err(|e| format!("读取课表数据失败: {}", e))?;


        // 使用新的 JSON parser 解析课表
        use crate::parser::parse_cufe_json;
        parse_cufe_json(&json_text)
    }

    /// 获取用户个人信息 (CUFE)
    /// 参考 SDK user_info.py 的实现
    pub async fn get_user_info(&self) -> Result<crate::models::UserInfo, String> {
        let url = format!("{}/xsxxxggl/xsgrxxwh_cxXsgrxx.html", self.api_base_url());

        let params = [
            ("gnmkdm", "N100801"),
            ("layout", "default"),
        ];


        // reqwest 的 cookie_store 会自动携带之前登录时保存的 cookies
        let response = self.client.get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("获取用户信息请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("获取用户信息失败: HTTP {}", response.status()));
        }

        let html_content = response.text().await.map_err(|e| format!("读取用户信息失败: {}", e))?;
        
        // 解析 HTML 获取用户基本信息（同步）
        let mut user_info = self.parse_user_info_html(&html_content)?;
        
        // 异步获取照片
        if !user_info.student_number.is_empty() {
            match self.fetch_photo_base64(&user_info.student_number).await {
                Ok(base64_data) => {
                    user_info.photo_url = Some(base64_data);
                },
                Err(_e) => {
                }
            }
        }
        
        Ok(user_info)
    }

    /// 解析用户信息 HTML (同步，不含照片)
    fn parse_user_info_html(&self, html: &str) -> Result<crate::models::UserInfo, String> {
        let document = Html::parse_document(html);
        
        // 定义选择器
        fn extract_text(doc: &Html, selector_str: &str) -> String {
            let selector = Selector::parse(selector_str).unwrap();
            doc.select(&selector)
                .next()
                .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
                .unwrap_or_default()
        }

        // 根据 SDK 的选择器提取信息
        // 学号和姓名在 panel-heading 中
        let student_number = extract_text(&document, "#ajaxForm > div > div.panel-heading > div > div:nth-child(1) > div > div > p");
        let name = extract_text(&document, "#ajaxForm > div > div.panel-heading > div > div:nth-child(2) > div > div > p");
        
        // 其他信息在表单中
        let department = extract_text(&document, "#col_jg_id > p");
        let class_name = extract_text(&document, "#col_bh_id > p");
        let grade = extract_text(&document, "#col_njdm_id > p");
        let major = extract_text(&document, "#col_zyfx_id > p");
        let gender = extract_text(&document, "#col_xbm > p");


        // 如果核心信息都为空，可能是登录失效
        if name.is_empty() && student_number.is_empty() {
            return Err("无法获取用户信息，可能登录已失效".to_string());
        }

        Ok(crate::models::UserInfo {
            student_number,
            name,
            department,
            class_name,
            grade,
            major,
            gender,
            photo_url: None, // 照片在调用处单独获取
        })
    }

    /// 获取照片并转换为 base64
    async fn fetch_photo_base64(&self, student_number: &str) -> Result<String, String> {
        use base64::{Engine as _, engine::general_purpose};

        // CUFE 照片 API URL
        let url = format!("{}/xtgl/photo_cxXszp4.html?xh_id={}&zplx=rxhzp", self.api_base_url(), student_number);


        // reqwest 的 cookie_store 会自动携带之前登录时保存的 cookies
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("获取照片请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("获取照片失败: HTTP {}", response.status()));
        }

        // 获取 Content-Type
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();

        // 获取图片数据
        let bytes = response.bytes().await
            .map_err(|e| format!("读取照片数据失败: {}", e))?;

        if bytes.is_empty() {
            return Err("照片数据为空".to_string());
        }

        // 转换为 base64
        let base64_data = general_purpose::STANDARD.encode(&bytes);
        
        // 构造 data URI
        let mime_type = if content_type.contains("png") {
            "image/png"
        } else if content_type.contains("gif") {
            "image/gif"
        } else {
            "image/jpeg"
        };
        
        let data_uri = format!("data:{};base64,{}", mime_type, base64_data);


        Ok(data_uri)
    }

    /// ============================================================
    /// 考试信息查询方法
    /// ============================================================

    /// 获取考试安排数据 (CUFE)
    /// 参数：
    /// - year: 学年名称，如 "2025-2026"
    /// - term: 学期 (1, 2, 3)
    pub async fn get_exam_schedule(&self, year: &str, term: i32) -> Result<serde_json::Value, String> {
        // 不需要 username 参数，但保持检查以确保已登录
        if self.username.is_none() {
            return Err("获取考试需要学号信息，请重新登录".to_string());
        }

        let url = format!("{}/kwgl/kscx_cxXsksxxIndex.html", self.api_base_url());

        // 学期码映射 (参考课表查询的映射规则)
        let term_code = match term {
            1 => "3",
            2 => "12",
            3 => "16",
            _ => "3",
        };

        // 查询参数
        let query_params = [
            ("doType", "query"),
            ("gnmkdm", "N358105"),
        ];

        // POST 数据
        let form_params = [
            ("xnm", year),
            ("xqm", term_code),
            ("_search", "false"),
            ("queryModel.showCount", "500"),
            ("queryModel.currentPage", "1"),
            ("queryModel.sortName", ""),
            ("queryModel.sortOrder", "asc"),
        ];


        let response = self.client.post(&url)
            .query(&query_params)
            .form(&form_params)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .map_err(|e| format!("获取考试请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("获取考试失败: HTTP {}", response.status()));
        }

        let json_text = response.text().await.map_err(|e| format!("读取考试数据失败: {}", e))?;


        // 解析 JSON
        serde_json::from_str(&json_text)
            .map_err(|e| format!("解析考试JSON失败: {}", e))
    }
}
