use crate::models::user_storefront::UserStorefront;
use reqwest::header::HeaderMap;
use reqwest::{header, Client};

#[derive(Debug, Clone)]
pub struct Request {
    authorization: String,
    user_token: String,
    storefront: String,
}

impl Request {
    pub(crate) fn new(authorization: String, user_token: String) -> Self {
        Self {
            authorization,
            user_token,
            storefront: String::new(),
        }
    }

    pub(crate) fn create_header(&mut self) -> HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert("origin", "https://music.apple.com".parse().unwrap());
        headers.insert("Authorization", self.authorization.parse().unwrap());
        headers.insert("Media-User-Token", self.user_token.parse().unwrap());
        headers
    }

    pub(crate) async fn get_user_storefront(&mut self) {
        let headers = self.create_header();
        let client = Client::builder().default_headers(headers).build().unwrap();

        let res = client
            .get("https://api.music.apple.com/v1/me/storefront")
            .send()
            .await
            .unwrap();
        let res_string = res.text().await.unwrap();
        let user_storefront: UserStorefront = serde_json::from_str(&res_string).unwrap();
        self.storefront = match user_storefront.data.first() {
            Some(data) => data.id.to_owned(),
            None => {
                panic!("Can't found user storefront")
            }
        }
    }

    pub(crate) fn get_song_id(url: &str) -> String {
        let mut match_header: i8 = 0;
        let mut id: String = String::new();
        for c in url.chars() {
            if match_header == 2 {
                match c.to_digit(10) {
                    Some(_) => {
                        id.push(c);
                    }
                    None => {
                        match_header = 0;
                        continue;
                    }
                }
            }
            if match_header == 1 && c == '=' {
                match_header = 2;
            } else if c == 'i' {
                match_header = 1;
            }
        }
        id
    }

    pub(crate) fn create_lyrics_url(&self, song_id: &str) -> String {
        format!("https://amp-api.music.apple.com/v1/catalog/{}/songs/{}?include[songs]=albums,lyrics,syllable-lyrics", self.storefront, song_id)
    }

    pub(crate) fn create_search_url(&self, song_name: &str, artist_name: &str) -> String {
        format!("https://amp-api-edge.music.apple.com/v1/catalog/{}/search?limit=5&platform=web&term={}&with=serverBubbles&types=songs%2Cactivities", self.storefront, format!("{} {}", song_name, artist_name))
    }
}

#[cfg(test)]
mod test {
    use crate::services::apple_music_url::Request;

    #[test]
    fn get_song_id_test() {
        let url_a = "https://music.apple.com/hk/album/%e7%84%a1%e7%ad%94%e6%a1%88/1729188120?i=1729188121&l=en-gb";
        let url_b = "https://music.apple.com/hk/album/%e7%84%a1%e7%ad%94%e6%a1%88/1729188120?i=1729188121&l=zh-hant-tw";
        assert_eq!("1729188121", Request::get_song_id(url_a));
        assert_eq!("1729188121", Request::get_song_id(url_b));
    }

    #[test]
    fn get_song_id_test_empty() {
        assert_eq!("", Request::get_song_id(""));
        assert_eq!("", Request::get_song_id("https://music.apple.com/hk/album/%e7%84%a1%e7%ad%94%e6%a1%88/1729188120?i=&l=zh-hant-tw"));
        assert_eq!("", Request::get_song_id("https://music.apple.com/hk/album/%e7%84%a1%e7%ad%94%e6%a1%88/1729188120?l=zh-hant-tw"));
    }
}
