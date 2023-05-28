use anyhow::{Context, Result};
use reqwest::{header, header::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, env};
pub struct Client {
    base_client: reqwest::Client,
    client_id: String,
    client_secret: String,
    username: String,
    password: String,
    subreddit: String,
}
// This is what redit calls a "thing"
#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub link_url: String,
    pub link_permalink: String,
    // This field is actually "link_id",
    // is one of the many identifiers for a "thing",
    // (see https://www.reddit.com/dev/api/#fullnames)
    // but I'm going to use it as an identifier
    // for each post.
    // Can't use 'id' because serde complains.
    #[serde(alias = "link_id")]
    pub post_id: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PostData {
    data: Post,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ModQueueJson {
    children: Vec<PostData>,
}

type ModQueue = Vec<Post>;
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
        let subreddit = env::var("SUBREDDIT").unwrap();
        Client {
            base_client: client,
            password,
            client_id,
            client_secret,
            username,
            subreddit,
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
            .with_context(|| {
                "Failed to get token, check your credentials or that reddit isn't down"
            })?
            .text()
            .await
            .expect("Empty response from token, something went awfully wrong");
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

        let access_token: String = response
            .get("access_token")
            .expect("Missing access token, cannot continue")
            .as_str()
            .unwrap()
            .into();

        Ok(access_token)
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

    pub async fn get_request(&mut self, endpoint: &str) -> Result<serde_json::Value> {
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

    pub async fn approve_post(&self, post_id: &str) -> Result<()> {
        let token = self.get_token().await?;
        let json_body = HashMap::from([("id", post_id)]);
        let response = self
            .base_client
            .post(format!("https://oauth.reddit.com/api/approve"))
            .headers(Client::construct_headers(&token))
            .json(&json_body)
            .send()
            .await?;
        let json = response.json().await?;
        println!(
            "Received json as response: {:?}",
            serde_json::to_string_pretty(&json)
        );
        Ok(())
    }

    pub async fn remove_post(&self, post_id: &str) -> Result<()> {
        let token = self.get_token().await?;
        let json_body = HashMap::from([("id", post_id)]);
        let response = self
            .base_client
            .post(format!("https://oauth.reddit.com/api/remove"))
            .headers(Client::construct_headers(&token))
            .json(&json_body)
            .send()
            .await?;
        println!("Received response: {response:?}");
        let json = response.json().await?;
        println!(
            "Received json as response: {}",
            serde_json::to_string_pretty(&json)?
        );
        Ok(())
    }

    pub async fn get_modqueue(&mut self, limit: u64) -> Result<Option<ModQueue>> {
        let sub = &(self.subreddit);
        let url = format!("/r/{sub}/about/modqueue?limit={limit}");
        let mut response_json = self.get_request(&url).await?;
        let posts_without_moderation: ModQueue =
            serde_json::from_value::<ModQueueJson>(response_json["data"].take())?
                .children
                .into_iter()
                .map(|PostData { data }| data)
                .collect();
        match posts_without_moderation[..] {
            [] => Ok(None),
            _ => Ok(Some(posts_without_moderation)),
        }
    }
}
