mod models;
mod services;

use core::str;
use std::sync::Arc;
use services::apple_music_url::Request;

use structopt::StructOpt;
use axum::{self, extract::{FromRef, Json, State}, http::StatusCode, response::{IntoResponse, Response}, routing::{get, post}, Router};
use tokio::{net::TcpListener, sync::Mutex};
use serde::{Deserialize, Serialize};
use services::token_handler::Token;
use reqwest::Client;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct NowListening {
    is_playing: bool,
    name: Option<String>,
    duration: Option<u64>,
    artist: Option<Vec<String>>,
    album: Option<String>,
    start_time: Option<u128>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
enum PlayStatus {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AutoUpdate {
    play_status: PlayStatus,
    name: Option<String>,
    artist: Option<String>
}

#[derive(structopt::StructOpt)]
struct Input {
    #[structopt(short, long, default_value = "0.0.0.0:3939")]
    address: String,
}

#[derive(Debug, Clone)]
struct ShareState {
    now_listening: Arc<Mutex<NowListening>>,
    request: Arc<Mutex<Request>>,
}

impl FromRef<ShareState> for Arc<Mutex<NowListening>> {
    fn from_ref(share_state: &ShareState) -> Self {
        share_state.now_listening.clone()
    }
}

impl FromRef<ShareState> for Arc<Mutex<Request>> {
    fn from_ref(share_state: &ShareState) -> Self {
        share_state.request.clone()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind(Input::from_args().address).await?;
    let now_listening = Arc::new(Mutex::new(NowListening {
        is_playing: false,
        name: None,
        duration: None,
        artist: None,
        album: None,
        start_time: None,
    }));

    println!("Loading apple music access token...");
    let authorization = Token::get_access_token().await.unwrap();
    println!("Access token: Done!");

    println!("Loading user token...");
    let user_token = Token::get_user_token();
    println!("User token: Done!");
    let mut request = Request::new(authorization, user_token);

    println!("Get user storefront...");
    request.get_user_storefront().await;
    println!("User storefront: Done!");

    let request = Arc::new(Mutex::new(request));

    let state = ShareState {
        now_listening: now_listening.clone(),
        request,
    };

    let app = Router::new()
        .route("/update", post(update))
        .route("/status", get(get_status))
        .route("/auto_update", post(auto_update))
        .with_state(state);

    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn update(
    State(state): State<Arc<Mutex<NowListening>>>,
    Json(payload): Json<NowListening>,
) -> Result<String> {
    let mut now_listening = state.lock().await;
    now_listening.is_playing = payload.is_playing;
    now_listening.name = payload.name;
    now_listening.duration = payload.duration;
    now_listening.artist = payload.artist;
    now_listening.album = payload.album;
    println!("Updated: {:?}", now_listening);
    Ok("Updated".to_string())
}

async fn auto_update(
    State(state): State<ShareState>,
    Json(payload): Json<AutoUpdate>,
) -> Result<String> {
    let mut now_listening = state.now_listening.lock().await;
    let mut request = state.request.lock().await;
    let headers = request.create_header();
    let client = Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    println!("Auto update: {:?}", payload);

    match payload.play_status {
        PlayStatus::Stopped => {
            now_listening.is_playing = false;
            now_listening.name = None;
            now_listening.duration =  None;
            now_listening.artist = None;
            now_listening.album = None;
            now_listening.start_time = None;
            return Ok("Not playing".to_string());
        },
        PlayStatus::Playing => {
            let url = request.create_search_url(payload.name.clone().unwrap().as_str(), payload.artist.clone().unwrap().as_str());

            let res = match client.get(url).send().await {
                Ok(r) => {
                    if r.status() != 200 {
                        panic!("Invalid response: {:}", r.text().await.unwrap());
                    }
                    r
                },
                Err(error) => {
                    panic!("{:}", error.to_string()); 
                }
            };
            let res_json: serde_json::Value = match res.json().await {
                Ok(json) => json,
                Err(error) => {
                    panic!("Failed to parse JSON: {}", error.to_string());
                }
            };

            now_listening.is_playing = true;
            now_listening.name = Some(payload.name.unwrap());
            now_listening.artist = Some(vec![payload.artist.unwrap()]);
            now_listening.album = res_json["results"]["top"]["data"][0]["attributes"]["albumName"]
                .as_str().map(|s| s.to_string());
            now_listening.duration = res_json["results"]["top"]["data"][0]["attributes"]["durationInMillis"]
                .as_u64();
            now_listening.start_time = Some(chrono::Local::now().timestamp_millis() as u128);
        },
        PlayStatus::Paused => {
            now_listening.is_playing = false;
            return Ok("Paused".to_string());
        }
    }

    

    // println!("Updated: {:?}", payload);
    Ok("Updated".to_string())
}

async fn get_status(State(state): State<Arc<Mutex<NowListening>>>) -> Result<Json<NowListening>> {
    let now_listening = state.lock().await;
    println!("Get status: {:?}", now_listening);
    Ok(Json(now_listening.clone()))
}

#[derive(Debug)]
pub struct Error(anyhow::Error);
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("VocasyncError: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
