use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::models::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIConfig {
    pub api_type: String,
    pub api_url: String,
    pub api_key: String,
    pub detection_model: String,
    pub report_model: String,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            api_type: "OpenAI Compatible".to_string(),
            api_url: "https://api.openai.com/v1".to_string(),
            api_key: "".to_string(),
            detection_model: "gpt-3.5-turbo".to_string(),
            report_model: "gpt-4-turbo-preview".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APITestResult {
    pub success: bool,
    pub message: String,
    pub response_time_ms: u64,
    pub model_used: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: Option<u64>,
    pub owned_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

pub struct AIService {
    config: AIConfig,
    client: reqwest::Client,
}

impl AIService {
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// 测试API连接
    pub async fn test_api_connection(&self) -> Result<APITestResult> {
        let start_time = std::time::Instant::now();
        
        if self.config.api_key.is_empty() {
            return Ok(APITestResult {
                success: false,
                message: "API Key不能为空".to_string(),
                response_time_ms: 0,
                model_used: None,
            });
        }

        println!("🧪 开始测试API连接 - 类型: {}", self.config.api_type);
        println!("📡 测试URL: {}", self.config.api_url);
        
        // 根据API类型选择不同的测试方式
        match self.config.api_type.as_str() {
            "OpenAI Compatible" => self.test_openai_connection(start_time).await,
            "Ollama (本地)" => self.test_ollama_connection(start_time).await,
            "Claude API" => self.test_claude_connection(start_time).await,
            _ => Ok(APITestResult {
                success: false,
                message: format!("不支持的API类型: {}", self.config.api_type),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                model_used: None,
            }),
        }
    }

    /// 测试OpenAI兼容API连接
    async fn test_openai_connection(&self, start_time: std::time::Instant) -> Result<APITestResult> {
        println!("🔌 测试OpenAI兼容API连接...");
        
        let test_url = format!("{}/models", self.config.api_url);
        
        let response = self.client
            .get(&test_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await;

        let elapsed = start_time.elapsed().as_millis() as u64;

        match response {
            Ok(resp) => {
                println!("📨 OpenAI API测试响应状态: {}", resp.status());
                if resp.status().is_success() {
                    match resp.json::<ModelsResponse>().await {
                        Ok(models) => {
                            println!("✅ OpenAI API连接成功，找到 {} 个模型", models.data.len());
                            Ok(APITestResult {
                                success: true,
                                message: format!("连接成功！找到 {} 个可用模型", models.data.len()),
                                response_time_ms: elapsed,
                                model_used: None,
                            })
                        },
                        Err(_) => {
                            println!("⚠️ OpenAI API连接成功，但无法解析模型列表");
                            Ok(APITestResult {
                                success: true,
                                message: "连接成功，但无法解析模型列表".to_string(),
                                response_time_ms: elapsed,
                                model_used: None,
                            })
                        }
                    }
                } else {
                    let status = resp.status();
                    let error_text = resp.text().await.unwrap_or_default();
                    println!("❌ OpenAI API测试失败: {} - {}", status, error_text);
                    Ok(APITestResult {
                        success: false,
                        message: format!("API返回错误: {} - {}", status, error_text),
                        response_time_ms: elapsed,
                        model_used: None,
                    })
                }
            }
            Err(e) => {
                println!("❌ OpenAI API网络连接失败: {}", e);
                Ok(APITestResult {
                    success: false,
                    message: format!("连接失败: {}", e),
                    response_time_ms: elapsed,
                    model_used: None,
                })
            }
        }
    }

    /// 测试Ollama本地API连接
    async fn test_ollama_connection(&self, start_time: std::time::Instant) -> Result<APITestResult> {
        println!("🔌 测试Ollama本地API连接...");
        
        // Ollama的API端点通常不需要认证，直接测试模型列表
        let test_url = format!("{}/api/tags", self.config.api_url.replace("/v1", ""));
        
        let response = self.client
            .get(&test_url)
            .header("Content-Type", "application/json")
            .send()
            .await;

        let elapsed = start_time.elapsed().as_millis() as u64;

        match response {
            Ok(resp) => {
                println!("📨 Ollama API测试响应状态: {}", resp.status());
                if resp.status().is_success() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            let model_count = json.get("models")
                                .and_then(|m| m.as_array())
                                .map(|arr| arr.len())
                                .unwrap_or(0);
                            
                            println!("✅ Ollama API连接成功，找到 {} 个模型", model_count);
                            Ok(APITestResult {
                                success: true,
                                message: format!("Ollama连接成功！找到 {} 个本地模型", model_count),
                                response_time_ms: elapsed,
                                model_used: None,
                            })
                        },
                        Err(_) => {
                            println!("⚠️ Ollama API连接成功，但响应格式异常");
                            Ok(APITestResult {
                                success: true,
                                message: "Ollama连接成功，但无法解析模型列表".to_string(),
                                response_time_ms: elapsed,
                                model_used: None,
                            })
                        }
                    }
                } else {
                    let status = resp.status();
                    let error_text = resp.text().await.unwrap_or_default();
                    println!("❌ Ollama API测试失败: {} - {}", status, error_text);
                    Ok(APITestResult {
                        success: false,
                        message: format!("Ollama API错误: {} - 请确认Ollama服务已启动", status),
                        response_time_ms: elapsed,
                        model_used: None,
                    })
                }
            }
            Err(e) => {
                println!("❌ Ollama API网络连接失败: {}", e);
                Ok(APITestResult {
                    success: false,
                    message: format!("Ollama连接失败: {} - 请检查Ollama是否运行在 {}", e, self.config.api_url),
                    response_time_ms: elapsed,
                    model_used: None,
                })
            }
        }
    }

    /// 测试Claude API连接
    async fn test_claude_connection(&self, start_time: std::time::Instant) -> Result<APITestResult> {
        println!("🔌 测试Claude API连接...");
        
        // Claude API没有直接的模型列表端点，我们发送一个简单的测试请求
        let test_body = serde_json::json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 10,
            "messages": [
                {
                    "role": "user",
                    "content": "Hi"
                }
            ]
        });

        let response = self.client
            .post(&format!("{}/messages", self.config.api_url))
            .header("x-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&test_body)
            .send()
            .await;

        let elapsed = start_time.elapsed().as_millis() as u64;

        match response {
            Ok(resp) => {
                println!("📨 Claude API测试响应状态: {}", resp.status());
                if resp.status().is_success() {
                    println!("✅ Claude API连接成功");
                    Ok(APITestResult {
                        success: true,
                        message: "Claude API连接成功！".to_string(),
                        response_time_ms: elapsed,
                        model_used: Some("claude-3-haiku-20240307".to_string()),
                    })
                } else {
                    let status = resp.status();
                    let error_text = resp.text().await.unwrap_or_default();
                    println!("❌ Claude API测试失败: {} - {}", status, error_text);
                    
                    let error_msg = if status == 401 {
                        "Claude API认证失败 - 请检查API密钥是否正确".to_string()
                    } else if status == 403 {
                        "Claude API访问被拒绝 - 请检查API密钥权限".to_string()
                    } else {
                        format!("Claude API错误: {} - {}", status, error_text)
                    };
                    
                    Ok(APITestResult {
                        success: false,
                        message: error_msg,
                        response_time_ms: elapsed,
                        model_used: None,
                    })
                }
            }
            Err(e) => {
                println!("❌ Claude API网络连接失败: {}", e);
                Ok(APITestResult {
                    success: false,
                    message: format!("Claude连接失败: {}", e),
                    response_time_ms: elapsed,
                    model_used: None,
                })
            }
        }
    }

    /// 获取可用模型列表
    pub async fn get_available_models(&self) -> Result<Vec<ModelInfo>> {
        if self.config.api_key.is_empty() {
            return Ok(vec![]);
        }

        let models_url = format!("{}/models", self.config.api_url);
        
        let response = self.client
            .get(&models_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status().is_success() {
            let models_response: ModelsResponse = response.json().await?;
            Ok(models_response.data)
        } else {
            Err(anyhow::anyhow!("获取模型列表失败: {}", response.status()))
        }
    }

    /// 使用检测模型分析应用使用模式
    pub async fn analyze_productivity(&self, activities: &[ApplicationActivity]) -> Result<f32> {
        if activities.is_empty() {
            return Ok(0.0);
        }

        // 这里应该调用配置的检测模型进行分析
        // 为了演示，暂时使用简单的计算
        let productive_time: u32 = activities
            .iter()
            .filter(|a| a.is_productive.unwrap_or(false))
            .map(|a| a.duration_seconds)
            .sum();

        let total_time: u32 = activities.iter().map(|a| a.duration_seconds).sum();

        if total_time == 0 {
            Ok(0.0)
        } else {
            Ok((productive_time as f32 / total_time as f32) * 100.0)
        }
    }

    /// 使用报告生成模型生成每日总结
    pub async fn generate_daily_summary(&self, 
        sessions: &[FocusSession], 
        _activities: &[ApplicationActivity]
    ) -> Result<String> {
        let total_focus_time: u32 = sessions
            .iter()
            .filter(|s| matches!(s.status, SessionStatus::Completed))
            .map(|s| s.duration_minutes)
            .sum();

        let completed_sessions = sessions
            .iter()
            .filter(|s| matches!(s.status, SessionStatus::Completed))
            .count();

        // 这里应该调用配置的报告生成模型
        // 为了演示，使用模板生成
        Ok(format!(
            "今日总结（使用模型: {}）：您完成了 {} 个专注会话，总专注时间 {} 分钟。继续保持专注，明天会更好！",
            self.config.report_model,
            completed_sessions,
            total_focus_time
        ))
    }

    pub async fn suggest_break_activities(&self) -> Result<Vec<String>> {
        Ok(vec![
            "站起来走动5分钟".to_string(),
            "做几个深呼吸练习".to_string(),
            "眺望远方放松眼睛".to_string(),
            "喝一杯水".to_string(),
            "做简单的颈部和肩部拉伸".to_string(),
        ])
    }

    pub async fn analyze_focus_patterns(&self, sessions: &[FocusSession]) -> Result<Vec<String>> {
        if sessions.is_empty() {
            return Ok(vec!["暂无足够的数据进行分析".to_string()]);
        }

        let completed_sessions = sessions
            .iter()
            .filter(|s| matches!(s.status, SessionStatus::Completed))
            .count();

        let total_sessions = sessions.len();
        let success_rate = (completed_sessions as f32 / total_sessions as f32) * 100.0;

        Ok(vec![
            format!("您的专注会话完成率为 {:.1}%（使用模型: {}）", success_rate, self.config.detection_model),
            if success_rate > 80.0 {
                "保持良好的专注习惯！".to_string()
            } else if success_rate > 60.0 {
                "可以尝试适当调整专注时长".to_string()
            } else {
                "建议从较短的专注时间开始".to_string()
            },
        ])
    }

    /// 分析内容（用于专注状态检测或报告生成）
    pub async fn analyze_content(&self, content: &str, model_type: &str) -> Result<String, String> {
        let model = match model_type {
            "detection" => &self.config.detection_model,
            "report" => &self.config.report_model,
            _ => return Err("不支持的模型类型".to_string()),
        };

        println!("🤖 准备调用AI API - 类型: {}", self.config.api_type);
        println!("📡 API URL: {}", self.config.api_url);
        println!("🎯 使用模型: {}", model);

        let client = reqwest::Client::new();
        
        // 根据API类型选择不同的调用方式
        match self.config.api_type.as_str() {
            "OpenAI Compatible" => self.call_openai_api(&client, content, model).await,
            "Ollama (本地)" => self.call_ollama_api(&client, content, model).await,
            "Claude API" => self.call_claude_api(&client, content, model).await,
            _ => Err(format!("不支持的API类型: {}", self.config.api_type)),
        }
    }

    /// 调用OpenAI兼容API
    async fn call_openai_api(&self, client: &reqwest::Client, content: &str, model: &str) -> Result<String, String> {
        println!("📞 调用OpenAI兼容API...");
        
        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": content
                }
            ],
            "max_tokens": 500,
            "temperature": 0.3
        });

        let response = client
            .post(&format!("{}/chat/completions", self.config.api_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await;

        self.parse_openai_response(response).await
    }

    /// 调用Ollama本地API
    async fn call_ollama_api(&self, client: &reqwest::Client, content: &str, model: &str) -> Result<String, String> {
        println!("📞 调用Ollama本地API...");
        
        let request_body = serde_json::json!({
            "model": model,
            "prompt": content,
            "stream": false
        });

        let response = client
            .post(&format!("{}/api/generate", self.config.api_url.replace("/v1", "")))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await;

        self.parse_ollama_response(response).await
    }

    /// 调用Claude API
    async fn call_claude_api(&self, client: &reqwest::Client, content: &str, model: &str) -> Result<String, String> {
        println!("📞 调用Claude API...");
        
        let request_body = serde_json::json!({
            "model": model,
            "max_tokens": 500,
            "messages": [
                {
                    "role": "user",
                    "content": content
                }
            ]
        });

        let response = client
            .post(&format!("{}/messages", self.config.api_url))
            .header("x-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await;

        self.parse_claude_response(response).await
    }

    /// 解析OpenAI格式响应
    async fn parse_openai_response(&self, response: Result<reqwest::Response, reqwest::Error>) -> Result<String, String> {
        match response {
            Ok(resp) => {
                println!("📨 OpenAI API响应状态: {}", resp.status());
                if resp.status().is_success() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                                if let Some(first_choice) = choices.first() {
                                    if let Some(message) = first_choice.get("message") {
                                        if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                                            println!("✅ OpenAI API调用成功，响应长度: {} 字符", content.len());
                                            return Ok(content.to_string());
                                        }
                                    }
                                }
                            }
                            println!("❌ OpenAI响应格式解析失败: {:?}", json);
                            Err("AI响应格式解析失败".to_string())
                        }
                        Err(e) => {
                            println!("❌ OpenAI JSON解析失败: {}", e);
                            Err(format!("JSON解析失败: {}", e))
                        }
                    }
                } else {
                    let status = resp.status();
                    let error_text = resp.text().await.unwrap_or_default();
                    println!("❌ OpenAI API请求失败: {} - {}", status, error_text);
                    Err(format!("API请求失败: {} - {}", status, error_text))
                }
            }
            Err(e) => {
                println!("❌ OpenAI网络请求失败: {}", e); 
                Err(format!("网络请求失败: {}", e))
            }
        }
    }

    /// 解析Ollama格式响应  
    async fn parse_ollama_response(&self, response: Result<reqwest::Response, reqwest::Error>) -> Result<String, String> {
        match response {
            Ok(resp) => {
                println!("📨 Ollama API响应状态: {}", resp.status());
                if resp.status().is_success() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Some(response_text) = json.get("response").and_then(|r| r.as_str()) {
                                println!("✅ Ollama API调用成功，响应长度: {} 字符", response_text.len());
                                return Ok(response_text.to_string());
                            }
                            println!("❌ Ollama响应格式解析失败: {:?}", json);
                            Err("Ollama响应格式解析失败".to_string())
                        }
                        Err(e) => {
                            println!("❌ Ollama JSON解析失败: {}", e);
                            Err(format!("Ollama JSON解析失败: {}", e))
                        }
                    }
                } else {
                    let status = resp.status();
                    let error_text = resp.text().await.unwrap_or_default();
                    println!("❌ Ollama API请求失败: {} - {}", status, error_text);
                    Err(format!("Ollama API请求失败: {} - {}", status, error_text))
                }
            }
            Err(e) => {
                println!("❌ Ollama网络请求失败: {}", e);
                Err(format!("Ollama网络请求失败: {}", e))
            }
        }
    }

    /// 解析Claude格式响应
    async fn parse_claude_response(&self, response: Result<reqwest::Response, reqwest::Error>) -> Result<String, String> {
        match response {
            Ok(resp) => {
                println!("📨 Claude API响应状态: {}", resp.status());
                if resp.status().is_success() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Some(content_array) = json.get("content").and_then(|c| c.as_array()) {
                                if let Some(first_content) = content_array.first() {
                                    if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                                        println!("✅ Claude API调用成功，响应长度: {} 字符", text.len());
                                        return Ok(text.to_string());
                                    }
                                }
                            }
                            println!("❌ Claude响应格式解析失败: {:?}", json);
                            Err("Claude响应格式解析失败".to_string())
                        }
                        Err(e) => {
                            println!("❌ Claude JSON解析失败: {}", e);
                            Err(format!("Claude JSON解析失败: {}", e))
                        }
                    }
                } else {
                    let status = resp.status();
                    let error_text = resp.text().await.unwrap_or_default();
                    println!("❌ Claude API请求失败: {} - {}", status, error_text);
                    Err(format!("Claude API请求失败: {} - {}", status, error_text))
                }
            }
            Err(e) => {
                println!("❌ Claude网络请求失败: {}", e);
                Err(format!("Claude网络请求失败: {}", e))
            }
        }
    }

    /// 生成专注状态分析的提示词
    fn build_monitoring_prompt(
        &self,
        app_name: &Option<String>,
        window_title: &Option<String>,
        ocr_text: &Option<String>,
        whitelist: &[String],
        blacklist: &[String],
        _activities: &[ApplicationActivity]
    ) -> String {
        let mut prompt = String::new();
        
        // 添加基础指令
        prompt.push_str("你是一个专注状态分析助手。基于以下信息，判断用户当前是专注(FOCUSED)还是分心(DISTRACTED)状态。\n\n");
        
        // 添加应用信息
        prompt.push_str("**当前应用信息：**\n");
        if let Some(app) = app_name {
            prompt.push_str(&format!("- 应用程序：{}\n", app));
        } else {
            prompt.push_str("- 应用程序：未检测到\n");
        }
        
        if let Some(title) = window_title {
            prompt.push_str(&format!("- 窗口标题：{}\n", title));
        } else {
            prompt.push_str("- 窗口标题：未检测到\n");
        }
        
        // 添加屏幕内容
        prompt.push_str("\n**屏幕内容：**\n");
        if let Some(text) = ocr_text {
            if text.len() > 1000 {
                prompt.push_str(&format!("{}...", &text[..1000]));
            } else {
                prompt.push_str(text);
            }
        } else {
            prompt.push_str("无可识别文本内容");
        }
        
        // 添加规则配置
        prompt.push_str("\n\n**判断规则：**\n");
        
        if !whitelist.is_empty() {
            prompt.push_str("专注应用白名单（以下应用视为专注状态）：\n");
            for app in whitelist {
                prompt.push_str(&format!("- {}\n", app));
            }
        }
        
        if !blacklist.is_empty() {
            prompt.push_str("分心应用黑名单（以下应用视为分心状态）：\n");
            for app in blacklist {
                prompt.push_str(&format!("- {}\n", app));
            }
        }
        
        // 添加输出格式要求
        prompt.push_str("\n**输出要求：**\n");
        prompt.push_str("请分析以上信息，并严格按照以下格式输出：\n");
        prompt.push_str("状态：FOCUSED 或 DISTRACTED\n");
        prompt.push_str("置信度：0.0-1.0之间的数值\n");
        prompt.push_str("原因：简要说明判断理由\n");
        
        prompt
    }
} 