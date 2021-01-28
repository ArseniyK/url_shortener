use async_trait::async_trait;
use validator::Validate;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Url {
    pub id: String,
    pub url: String,
    pub count: u64,
}

#[derive(Serialize)]
pub struct ResponseUrl {
    pub id: String,
    pub short_url: String,
    pub long_url: String,
    pub count: u64,
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct CreateUrl {
    #[validate(url)]
    pub url: String,
}

#[derive(Deserialize)]
pub struct RedirectParams {
    pub id: String,
}


#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UrlService {
    async fn shorten(&self, url: &str, user: &str) -> Option<Url>;
    async fn get(&self, id: &str) -> Option<Url>;
    async fn new_user(&self) -> Option<String>;
    async fn get_last_n_for_user(&self, user: &str, n:isize) -> Vec<Url>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UrlRepo {
    async fn generate(&self, url: &str) -> Option<Url>;
    async fn get(&self, id: &str) -> Option<Url>;
    async fn increment_counter(&self, id: &str) -> bool;
    async fn new_user(&self) -> Option<String>;
    async fn generate_for_user(&self, url: &str, user: &str) -> Option<Url>;
    async fn get_last_n_for_user(&self, user: &str, n:isize) -> Vec<Url>;
}
