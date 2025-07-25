use std::path::{Path, PathBuf};
use std::fs::{self, File, create_dir_all};
use std::io::{Read, Write};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::models::{Email, EmailAccount};

// 布局设置结构体
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LayoutSettings {
    pub window_width: Option<f64>,
    pub window_height: Option<f64>,
    pub sidebar_width: f64,
    pub email_list_width: f64,
    pub content_width: f64,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        // 默认布局设置
        LayoutSettings {
            window_width: Some(1024.0),
            window_height: Some(768.0),
            sidebar_width: 10.0,    // 左侧栏默认宽度
            email_list_width: 20.0, // 邮件列表默认宽度
            content_width: 70.0,    // 内容区默认宽度
        }
    }
}

// 记录邮件同步状态和应用设置的结构体
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AppState {
    pub last_sync: Option<DateTime<Utc>>,
    pub uid_map: HashMap<String, Vec<String>>, // 邮箱 -> 已同步的UID列表
    pub layout: LayoutSettings, // 添加布局设置
}

#[derive(Clone)]
pub struct LocalStorage {
    base_path: PathBuf,
    app_state: AppState,
}

impl LocalStorage {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 获取用户数据目录
        let base_path = dirs::data_local_dir()
            .ok_or("无法获取本地数据目录")?
            .join("RustMail");
        
        // 确保目录存在
        create_dir_all(&base_path)?;
        
        // 加载或创建应用状态
        let app_state = Self::load_app_state(&base_path).unwrap_or_default();
        
        Ok(Self {
            base_path,
            app_state,
        })
    }
    
    // 获取账户的邮件存储目录
    fn get_account_path(&self, account: &EmailAccount) -> PathBuf {
        let safe_address = account.address.replace("@", "_at_").replace(".", "_dot_");
        self.base_path.join(&safe_address)
    }
    
    // 获取指定文件夹的路径
    fn get_folder_path(&self, account: &EmailAccount, folder: &str) -> PathBuf {
        self.get_account_path(account).join(folder)
    }
    
    // 加载应用状态
    fn load_app_state(base_path: &Path) -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
        let state_file_path = base_path.join("app_state.json");
        
        if !state_file_path.exists() {
            return Ok(AppState::default());
        }
        
        let mut file = File::open(state_file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let state: AppState = serde_json::from_str(&contents)?;
        Ok(state)
    }
    
    // 保存应用状态
    pub fn save_app_state(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let state_file_path = self.base_path.join("app_state.json");
        let json = serde_json::to_string_pretty(&self.app_state)?;
        
        let mut file = File::create(state_file_path)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
    
    // 保存邮件到本地
    pub fn save_emails(&mut self, account: &EmailAccount, folder: &str, emails: &[Email]) 
        -> Result<(), Box<dyn std::error::Error + Send + Sync>> 
    {
        let folder_path = self.get_folder_path(account, folder);
        create_dir_all(&folder_path)?;
        
        // 获取该账户的UID映射，如果不存在则创建
        let account_key = account.address.clone();
        let uid_map = self.app_state.uid_map
            .entry(account_key)
            .or_insert_with(Vec::new);
            
        // 只保存新邮件
        for email in emails {
            if !uid_map.contains(&email.id) {
                let email_path = folder_path.join(format!("{}.json", &email.id));
                let json = serde_json::to_string_pretty(email)?;
                
                let mut file = File::create(email_path)?;
                file.write_all(json.as_bytes())?;
                
                // 添加到已同步列表
                uid_map.push(email.id.clone());
            }
        }
        
        // 更新同步状态
        self.app_state.last_sync = Some(Utc::now());
        self.save_app_state()?;
        
        Ok(())
    }
    
    // 从本地加载邮件
    pub fn load_emails(&self, account: &EmailAccount, folder: &str) 
        -> Result<Vec<Email>, Box<dyn std::error::Error + Send + Sync>> 
    {
        let folder_path = self.get_folder_path(account, folder);
        
        if !folder_path.exists() {
            return Ok(Vec::new());
        }
        
        let mut emails = Vec::new();
        
        for entry in fs::read_dir(folder_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "json") {
                let mut file = File::open(&path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                
                let email: Email = serde_json::from_str(&contents)?;
                emails.push(email);
            }
        }
        
        // 按日期排序，最新的邮件在前面
        emails.sort_by(|a, b| b.date.cmp(&a.date));
        
        Ok(emails)
    }
    
    // 获取最后同步时间 (兼容现有代码)
    pub fn get_last_sync(&self) -> Option<DateTime<Utc>> {
        self.app_state.last_sync
    }
    
    // 获取已同步的邮件ID列表
    pub fn get_synced_ids(&self, account: &EmailAccount) -> Vec<String> {
        self.app_state.uid_map
            .get(&account.address)
            .cloned()
            .unwrap_or_default()
    }
    
    // 获取布局设置文件路径
    fn get_layout_settings_path(&self) -> PathBuf {
        self.base_path.join("layout_settings.json")
    }
    
    // 保存布局设置
    pub fn save_layout_settings(&self, settings: &LayoutSettings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let layout_file = self.get_layout_settings_path();
        let json = serde_json::to_string_pretty(&settings)?;
        
        let mut file = File::create(layout_file)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
    
    // 加载布局设置
    pub fn load_layout_settings(&self) -> Result<LayoutSettings, Box<dyn std::error::Error + Send + Sync>> {
        let layout_file = self.get_layout_settings_path();
        
        if !layout_file.exists() {
            return Ok(LayoutSettings::default());
        }
        
        let mut file = File::open(layout_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let settings: LayoutSettings = serde_json::from_str(&contents)?;
        Ok(settings)
    }
}