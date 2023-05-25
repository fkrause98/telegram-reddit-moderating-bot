use anyhow::Result;
use reqwest::header;
use serde_json::Value;
use std::env;
async fn oauth_request(
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
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client
        .post("https://www.reddit.com/api/v1/access_token")
        .basic_auth(client_id, Some(client_secret))
        .headers(headers)
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
    println!("The response: {:?}", json);
    Ok(json)
}
pub async fn get_token() -> Result<String> {
    let client_id = env::var("CLIENT_ID")?;
    let client_secret = env::var("CLIENT_SECRET")?;
    let username = env::var("USERNAME")?;
    let password = env::var("PASSWORD")?;

    let response = oauth_request(&client_id, &client_secret, &username, &password).await?;
    let access_token = response
        .get("access_token")
        .expect("Missing access token, cannot continue")
        .as_str()
        .unwrap();

    println!("The access token: {:?}", access_token);
    return Ok(access_token.to_string());
}
