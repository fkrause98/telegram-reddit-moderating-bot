use anyhow::Result;
use reqwest::{header, header::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::env;
pub struct Client {
    base_client: reqwest::Client,
    client_id: String,
    client_secret: String,
    username: String,
    password: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RedditPost {
    pub link_url: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RedditPostData {
    data: RedditPost,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ModQueueJson {
    children: Vec<RedditPostData>,
}
type ModQueue = Vec<RedditPost>;
impl Client {
    pub fn new() -> Client {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let client_id = env::var("CLIENT_ID").unwrap();
        let client_secret = env::var("CLIENT_SECRET").unwrap();
        let username = env::var("USERNAME").unwrap();
        let password = env::var("PASSWORD").unwrap();
        Client {
            base_client: client,
            password,
            client_id,
            client_secret,
            username,
        }
    }
    async fn oauth_request(
        &self,
        client_id: &str,
        client_secret: &str,
        username: &str,
        password: &str,
    ) -> Result<Value> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        headers.insert(
            "User-Agent",
            "telegram:fran:v1.4.0 (by /u/The_L_Of_Life)"
                .parse()
                .unwrap(),
        );
        let response = self
            .base_client
            .post("https://www.reddit.com/api/v1/access_token")
            .headers(headers)
            .basic_auth(client_id, Some(client_secret))
            .body(format!(
                "grant_type=password&username={username}&password={password}"
            ))
            .send()
            .await
            .expect("Error sending request")
            .text()
            .await
            .unwrap();
        let json = serde_json::from_str(&response).unwrap();
        Ok(json)
    }
    pub async fn get_token(&self) -> Result<String> {
        let response = self
            .oauth_request(
                &self.client_id,
                &self.client_secret,
                &self.username,
                &self.password,
            )
            .await?;
        let access_token = response
            .get("access_token")
            .expect("Missing access token, cannot continue")
            .as_str()
            .unwrap();

        Ok(access_token.to_string())
    }

    fn construct_headers(token: &str) -> HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "User-Agent",
            "telegram:fran:v1.4.0 (by /u/The_L_Of_Life)"
                .parse()
                .unwrap(),
        );
        headers.insert("Authorization", format!("bearer {token}").parse().unwrap());
        headers
    }

    pub async fn reddit_request(&mut self, endpoint: &str) -> Result<serde_json::Value> {
        let token = self.get_token().await?;
        let response = self
            .base_client
            .get(format!("https://oauth.reddit.com/{endpoint}"))
            .headers(Client::construct_headers(&token))
            .send()
            .await?;
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    }
    pub async fn get_modqueue(&mut self, limit: u64) -> Result<Option<ModQueue>> {
        let url = format!("/r/dankgentina/about/modqueue?limit={limit}");
        let mut response_json = self.reddit_request(&url).await?;
        let posts_without_moderation: ModQueue =
            serde_json::from_value::<ModQueueJson>(response_json["data"].take())?
                .children
                .into_iter()
                .map(|RedditPostData { data }| data)
                .collect();
        match posts_without_moderation[..] {
            [] => Ok(None),
            _ => Ok(Some(posts_without_moderation)),
        }
    }
}
