use fancy_regex::Regex;
use serde_yaml::Value;
use std::fs;
use std::{
    fs::File,
    io::{stdin, Write},
};

pub struct Token {}

impl Token {
    pub(crate) fn get_user_token() -> String {
        let config_path = "config.yml";

        if !std::path::Path::new(config_path).exists() {
            File::create(config_path).expect("Unable to create config file");
        }

        let config_content = fs::read_to_string(config_path).expect("Unable to read config file");
        let config: Value =
            serde_yaml::from_str(&config_content).expect("Unable to parse config file");

        match config["user_token"].as_str() {
            Some(token) => token.to_string(),
            None => {
                println!("user_token not found in config, please enter user token:");
                Token::check_user_input()
            }
        }
    }

    pub(crate) fn check_user_input() -> String {
        let mut user_input = String::new();
        println!("Enter user token: ");
        stdin().read_line(&mut user_input).unwrap();
        user_input = match user_input.strip_suffix('\n') {
            Some(result) => result.to_string(),
            None => user_input,
        };
        if user_input.is_empty() {
            panic!("user token is empty");
        }
        user_input
    }

    /// Get apple access token form web ui
    pub(crate) async fn get_access_token() -> Result<String, String> {
        let res = reqwest::get("https://music.apple.com")
            .await
            .map_err(|error| error.to_string())?;
        if res.status().as_u16() != 200 {
            Err("Unable to get music.apple.com".to_string())?
        }
        let res_text = res.text().await.map_err(|error| error.to_string())?;
        let js_re = Regex::new(r#"(?<=index)(.*?)(?=\.js")"#).unwrap();
        let js_file = js_re.find(&res_text).unwrap().unwrap().as_str();
        println!("{:#?}", js_file);
        let js_res = reqwest::get(format!("https://music.apple.com/assets/index{js_file}.js"))
            .await
            .unwrap();
        if js_res.status().as_u16() != 200 {
            Err("Unable to get js file".to_string())?
        }
        let js_res_text = js_res.text().await.unwrap();
        // println!("{:#?}", js_res_text);
        let token_re = Regex::new(r#"(?=eyJh)(.*?)(?=")"#).unwrap();
        let token = match token_re.find(&js_res_text) {
            Ok(data) => match data {
                Some(value) => value.as_str(),
                None => Err("Token is empty".to_string())?,
            },
            Err(error) => Err(error.to_string())?,
        };
        Ok(format!("Bearer {token}"))
    }
}
