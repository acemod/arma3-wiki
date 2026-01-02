use std::path::PathBuf;

use reqwest::{
    Client, RequestBuilder,
    cookie::{CookieStore, Jar},
};

pub fn temp() -> PathBuf {
    std::env::temp_dir().join("arma3-wiki-fetch/commands")
}

pub trait WafSkip {
    fn bi_get(&self, url: &str) -> RequestBuilder;
    fn bi_post(&self, url: &str) -> RequestBuilder;
    fn bi_head(&self, url: &str) -> RequestBuilder;
}

#[derive(Debug, serde::Deserialize)]
pub struct CommunityDetails {
    session: String,
    token: String,
    user_id: String,
    user_name: String,
    shopify: String,
}

impl CommunityDetails {
    pub fn load() -> Self {
        serde_yaml::from_str::<CommunityDetails>(
            &std::fs::read_to_string(PathBuf::from(".community_details.yaml"))
                .expect("Failed to read community details"),
        )
        .expect("Failed to parse community details")
    }

    pub fn to_cookies(&self) -> Jar {
        let jar = Jar::default();
        let url = "https://community.bistudio.com/".parse().unwrap();
        jar.add_cookie_str(&format!("community_session={}", self.session), &url);
        jar.add_cookie_str(&format!("communityToken={}", self.token), &url);
        jar.add_cookie_str(&format!("communityUserID={}", self.user_id), &url);
        jar.add_cookie_str(&format!("communityUserName={}", self.user_name), &url);
        jar.add_cookie_str(&format!("_shopify_y={}", self.shopify), &url);
        println!("Cookies: {:?}", jar.cookies(&url));
        jar
    }
}

impl WafSkip for Client {
    fn bi_get(&self, url: &str) -> RequestBuilder {
        self.get(url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0",
            )
            .header(
                "bi-waf-skip",
                std::env::var("BI_WAF_SKIP").expect("BI_WAF_SKIP not set"),
            )
    }

    fn bi_post(&self, url: &str) -> RequestBuilder {
        self.post(url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0",
            )
            .header(
                "bi-waf-skip",
                std::env::var("BI_WAF_SKIP").expect("BI_WAF_SKIP not set"),
            )
            .header("Content-Type", "application/x-www-form-urlencoded")
    }

    fn bi_head(&self, url: &str) -> RequestBuilder {
        self.head(url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0",
            )
            .header(
                "bi-waf-skip",
                std::env::var("BI_WAF_SKIP").expect("BI_WAF_SKIP not set"),
            )
    }
}
