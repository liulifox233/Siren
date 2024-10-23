use fancy_regex::Regex;
use std::{fs::File, io::{stdin, Write}};

pub struct Token {}

impl Token {
    pub(crate) fn get_user_token() -> String {
        match dotenv::from_path("./.env") {
            Ok(_) => {
                dotenv::var("USER_TOKEN").unwrap()
            },
            Err(_) => {
                let user_input = Self::check_user_input();
                let file_user_token = format!(r#"USER_TOKEN="{}""#, user_input);
                let mut file = File::create(".env").unwrap();
                file.write_all(file_user_token.as_bytes()).expect("Unable write data to file");
                user_input
            }
        }
    }
    
    pub(crate) fn check_user_input() -> String {
        let mut user_input = String::new();
        println!("Enter user token: ");
        stdin().read_line(&mut user_input).unwrap();
        user_input = match user_input.strip_suffix('\n') {
            Some(result) => result.to_string(),
            None => user_input
        };
        if user_input.is_empty() {
            panic!("user token is empty");
        }
        user_input
    }
    
    /// Get apple access token form web ui
    pub(crate) async fn get_access_token() -> Result<String, String> {
        let res = reqwest::get("https://music.apple.com").await.map_err(|error| error.to_string())?;
        if res.status().as_u16() != 200 {
            Err("Unable to get music.apple.com".to_string())?
        }
        let res_text = res.text().await.map_err(|error| error.to_string())?;
        let js_re = Regex::new(r#"(?<=index)(.*?)(?=\.js")"#).unwrap();
        let js_file = js_re.find(&res_text).unwrap().unwrap().as_str();
        println!("{:#?}",js_file);
        let js_res = reqwest::get(format!("https://music.apple.com/assets/index{js_file}.js")).await.unwrap();
        if js_res.status().as_u16() != 200 {
            Err("Unable to get js file".to_string())?
        }
        let js_res_text = js_res.text().await.unwrap();
        // println!("{:#?}", js_res_text);
        let token_re = Regex::new(r#"(?=eyJh)(.*?)(?=")"#).unwrap();
        let token = match token_re.find(&js_res_text) {
            Ok(data) => {
                match data {
                    Some(value) => value.as_str(),
                    None => {
                        Err("Token is empty".to_string())?
                    }
                }
            },
            Err(error) => {
                Err(error.to_string())?
            }
        };
        Ok(format!("Bearer {token}"))
    }
}
