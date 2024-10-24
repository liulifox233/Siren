use std::{
    fs::File,
    io::{stdin, Write},
};

use anyhow::Result;
use axum::http::status;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::NowListening;
use serde_yaml::Value;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct FeishuRequest {
    app_id: String,
    app_secret: String,
    token: String,
    expire_time: u128,
    user_list: Vec<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserList {
    user_list: Vec<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    user_id: String,
    end_time: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body {
    system_status: SystemStatus,
    pub update_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemStatus {
    title: String,
    i18n_title: I18n,
    icon_key: String,
    color: String,
    priority: u8,
    sync_setting: SyncSetting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct I18n {
    en_us: String,
    zh_cn: String,
    ja_jp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncSetting {
    is_open_by_default: bool,
    title: String,
    i18n_title: I18n,
    explain: String,
    i18n_explain: I18n,
}

impl FeishuRequest {
    pub fn new() -> Self {
        let mut feishu_request = FeishuRequest {
            app_id: String::new(),
            app_secret: String::new(),
            token: String::new(),
            expire_time: 0,
            user_list: Vec::new(),
        };
        feishu_request.get_app_information();
        feishu_request.refresh_token();
        feishu_request
    }

    fn create_header(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.token).parse().unwrap(),
        );
        headers
    }

    fn create_client(&self) -> Client {
        Client::builder()
            .default_headers(self.create_header())
            .build()
            .unwrap()
    }

    pub async fn update_status(&self, now_listening: NowListening) {
        let client = self.create_client();
        let status_id = self.get_status().await;

        let body_str = r#"{
  "system_status": {
    "title": "拖延症候群",
    "i18n_title": {
      "zh_cn": "拖延症候群",
      "en_us": "Procrastination Syndrome",
      "ja_jp": "先延ばし症候群"
    },
    "icon_key": "GeneralMoonRest",
    "color": "VIOLET",
    "priority": 1,
    "sync_setting": {
      "is_open_by_default": true,
      "title": "出差期间自动开启",
      "i18n_title": {
        "zh_cn": "出差期间自动开启",
        "en_us": "Auto display Business Trip",
        "ja_jp": "出張中に自動的にオンにする"
      },
      "explain": "出差审批通过后，将自动开启并优先展示该状态。",
      "i18n_explain": {
        "zh_cn": "出差审批通过后，该状态将自动开启并优先展示",
        "en_us": "Auto-display after travel request is approved.",
        "ja_jp": "申請が承認されると、このステータスが優先的に表示されます"
      }
    }
  },
  "update_fields": [
    "ICON",
    "COLOR",
    "TITLE",
    "I18N_TITLE"
  ]
}"#;
        let mut body: Body = serde_json::from_str(body_str).unwrap();

        match now_listening.is_playing {
            true => {
                let name = now_listening.name.clone().unwrap();
                let mut char_count = 0;
                let mut truncated_name = String::new();

                for c in name.chars() {
                    if c.is_ascii() {
                        char_count += 1;
                    } else {
                        char_count += 2;
                    }

                    if char_count > 20 {
                        break;
                    }

                    truncated_name.push(c);
                }

                body.system_status.title = format!("🎵 {}", truncated_name);
                body.system_status.i18n_title.zh_cn =
                    format!("🎵 {}", now_listening.name.clone().unwrap());
                body.system_status.i18n_title.en_us =
                    format!("🎵 {}", now_listening.name.clone().unwrap());
                body.system_status.i18n_title.ja_jp = format!("🎵 {}", now_listening.name.unwrap());
            }
            _ => (),
        }

        let res = client
            .patch(format!(
                "https://open.feishu.cn/open-apis/personal_settings/v1/system_statuses/{}",
                status_id
            ))
            .json(&body)
            .send()
            .await
            .unwrap();
        let res_string = res.text().await.unwrap();
        println!("Feishu response: {:?}", res_string);
    }

    pub async fn set_status(&mut self, time: u128) {
        let client = self.create_client();
        let status_id = self.get_status().await;
        self.user_list = self
            .user_list
            .iter()
            .map(|x| User {
                user_id: x.user_id.clone(),
                end_time: time / 1000,
            })
            .collect();

        println!(
            "{:#?}",
            json!(UserList {
                user_list: self.user_list.clone()
            })
            .to_string()
        );

        let res = client
            .post(format!("https://open.feishu.cn/open-apis/personal_settings/v1/system_statuses/{}/batch_open", status_id))
            .json(&UserList{user_list: self.user_list.clone()})
            .send()
            .await
            .unwrap();
        println!("{:#?}", res.text().await.unwrap());
    }

    async fn get_status(&self) -> String {
        let client = self.create_client();
        let res = client
            .get("https://open.feishu.cn/open-apis/personal_settings/v1/system_statuses")
            .send()
            .await
            .unwrap();
        let res_json: serde_json::Value = res.json().await.unwrap();
        println!("{:#?}", res_json);
        let status_id = res_json["data"]["items"][0]["system_status_id"]
            .as_str()
            .unwrap()
            .to_string();
        status_id
    }

    pub async fn refresh_token(&mut self) {
        let now = chrono::Utc::now().timestamp_millis() as u128;
        if now < self.expire_time {
            return;
        }
        let client = Client::new();
        let res = client
            .post("https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal/")
            .json(&serde_json::json!({
                "app_id": self.app_id,
                "app_secret": self.app_secret,
            }))
            .send()
            .await
            .unwrap();
        let res_json: serde_json::Value = res.json().await.unwrap();
        self.token = res_json["tenant_access_token"]
            .as_str()
            .unwrap()
            .to_string();
        let now = chrono::Utc::now().timestamp_millis() as u128;
        self.expire_time = (res_json["expire"].as_u64().unwrap() as u128) * 1000 + now;
    }

    fn get_app_information(&mut self) {
        let mut file = File::open("config.yml").expect("Unable to open config file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read config file");
        let config: Value = serde_yaml::from_str(&contents).expect("Unable to parse config file");

        self.app_id = config["app_id"].as_str().unwrap().to_string();
        self.app_secret = config["app_secret"].as_str().unwrap().to_string();
        self.user_list = config["user_list"]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|x| User {
                user_id: x.as_str().unwrap().to_string(),
                end_time: 0,
            })
            .collect();
    }
}
