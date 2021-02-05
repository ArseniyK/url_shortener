use super::error::UrlError;
use crate::urls::utils::BuildUrl;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::vec::Vec;
use validator::Validate;

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

impl From<Url> for ResponseUrl {
    fn from(url: Url) -> Self {
        ResponseUrl {
            id: url.id.clone(),
            short_url: url.build_url(),
            long_url: url.url.clone(),
            count: url.count.clone(),
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct CreateUrl {
    #[validate(url(message = "Enter valid url"))]
    pub url: String,
}

#[derive(Deserialize)]
pub struct RedirectParams {
    pub id: String,
}

#[derive(Deserialize)]
pub struct PageParams {
    pub page: Option<isize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Paginated<T> {
    pub total: isize,
    pub page_count: isize,
    pub next: Option<isize>,
    pub prev: Option<isize>,
    pub results: Vec<T>,
}

impl From<Paginated<Url>> for Paginated<ResponseUrl> {
    fn from(paginated: Paginated<Url>) -> Self {
        Paginated {
            total: paginated.total,
            page_count: paginated.page_count,
            next: paginated.next,
            prev: paginated.prev,
            results: paginated
                .results
                .into_iter()
                .map(ResponseUrl::from)
                .collect(),
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UrlService {
    async fn shorten(&self, url: &str, user: &str) -> Result<Url, UrlError>;
    async fn get(&self, id: &str) -> Result<Url, UrlError>;
    async fn new_user(&self) -> Result<String, UrlError>;
    async fn get_urls_for_user(&self, user: &str, page: isize) -> Paginated<Url>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UrlRepo {
    async fn generate(&self, url: &str) -> Result<Url, UrlError>;
    async fn get(&self, id: &str) -> Result<Url, UrlError>;
    async fn increment_counter(&self, id: &str) -> Result<bool, UrlError>;
    async fn new_user(&self) -> Result<String, UrlError>;
    async fn generate_for_user(&self, url: &str, user: &str) -> Result<Url, UrlError>;
    async fn get_urls_for_user(&self, user: &str, start: isize, stop: isize) -> Vec<Url>;
    async fn count_urls_for_user(&self, user: &str) -> Result<isize, UrlError>;
}
