use super::types::*;
use async_trait::async_trait;
use redis::{AsyncCommands, RedisResult, RedisError};
use std::sync::Arc;
use harsh::Harsh;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::urls::error::UrlError;

const URL_COUNTER_KEY: &str = "url_shortener:url_counter";
const USER_COUNTER_KEY: &str = "url_shortener:user_counter";
const URLS_KEY: &str = "url_shortener:urls";
const USERS_KEY: &str = "url_shortener:users";


impl From<RedisError> for UrlError {
    fn from(_: RedisError) -> UrlError {
        UrlError {}
    }
}

pub struct RedisUrlRepoImpl {
    pub redis_client: Arc<redis::Client>,
    pub hashids: Harsh
}

impl RedisUrlRepoImpl {
    fn get_key(self: &Self, id: &str) -> String {
        format!("{}:{}", URLS_KEY, id)
    }

    fn get_user_key(self: &Self, id: &str) -> String {
        format!("{}:{}", USERS_KEY, id)
    }

    async fn get_next_key(self: &Self) -> Result<String, UrlError> {
        let redis_client = &*self.redis_client;
        redis_client.get_async_connection()
            .await?
            .incr(URL_COUNTER_KEY, 1)
            .await
            .map(|result|(self.hashids.encode(&[result,])))
            .map_err(|_| UrlError)
    }

    async fn save_for_user(self: &Self, id: &str, user: &str) {
        let redis_client = &*self.redis_client;
        if let Ok(mut conn) = redis_client.get_async_connection().await {
            let _:RedisResult<String> = conn.zadd(
                self.get_user_key(user),
                id,
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()
            ).await;
        };
    }
}

#[async_trait]
impl UrlRepo for RedisUrlRepoImpl {
    async fn generate(&self, url: &str) -> Result<Url, UrlError> {
        let redis_client = &*self.redis_client;
        let next_key = self.get_next_key().await.unwrap();
        let url_key = self.get_key(&next_key);
        let mut conn = redis_client.get_async_connection().await?;

        conn.hset_multiple(
            url_key, &[("id", &*next_key), ("url", &*url), ("count", "0")],
        ).await?;
        Ok(Url { id: next_key.clone(), url: url.parse().unwrap(), count: 0 })
    }

    async fn get(&self, id: &str) -> Result<Url, UrlError> {
        let redis_client = &*self.redis_client;
        let url_key = self.get_key(&id);
        let mut conn = redis_client.get_async_connection().await?;

        let res: RedisResult<Vec<String>> = conn.hget(
                url_key, &["id", "url", "count"],
            ).await;

        match res {
            Ok(result) => {
                if result.len() == 3 {
                    Ok(Url {
                        id: result[0].clone(),
                        url: result[1].clone(),
                        count: result[2].parse().unwrap(),
                    })
                } else {
                    Err(UrlError)
                }

            }
            _ => { Err(UrlError) }
        }
    }

    async fn increment_counter(&self, id: &str) -> Result<bool, UrlError> {
        let redis_client = &*self.redis_client;
        let url_key = self.get_key(&id);
        redis_client.get_async_connection()
            .await?
            .hincr(
                url_key, "count", 1
            ).await
            .map(|_: String| true)
            .map_err(|_| UrlError)
    }

    async fn new_user(&self) -> Result<String, UrlError> {
        let redis_client = &*self.redis_client;
        redis_client.get_async_connection().await?
            .incr(USER_COUNTER_KEY, 1)
            .await
            .map(|result| (self.hashids.encode(&[result, ])))
            .map_err(|_| UrlError)
    }

    async fn generate_for_user(&self, url: &str, user: &str) -> Result<Url, UrlError> {
        let url = self.generate(url).await;
        match url {
            Ok(url) => {
                self.save_for_user(&url.id.clone(), user).await;
                Ok(url)
            },
            Err(error) => Err(error)
        }
    }

    async fn get_last_n_for_user(&self, user: &str, n:isize) -> Vec<Url> {
        let redis_client = &*self.redis_client;
        let mut res = vec![];
        if let Ok(mut conn) = redis_client.get_async_connection().await {
            let ids:Vec<String> = conn.zrevrange(self.get_user_key(user), 0, n-1).await.unwrap_or(vec![]);
            for key in ids.into_iter() {
                if let Ok(url) = self.get(&*key).await {
                    res.push(url)
                }
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::hashids;

    async fn setup() -> RedisUrlRepoImpl {
        let redis_client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let hashids = hashids::configure().await;
        RedisUrlRepoImpl { redis_client: Arc::new(redis_client), hashids }
    }

    #[actix_web::main]
    #[test]
    async fn test_get_key() {
        let sut = setup().await;
        let key = sut.get_key("test_id");
        assert_eq!("url_shortener:urls:test_id", key);
    }

    #[actix_web::main]
    #[test]
    async fn test_next_key() {
        let sut = setup().await;
        let next_key_1 = sut.get_next_key().await.ok();
        let next_key_2 = sut.get_next_key().await.ok();
        assert_ne!(None, next_key_1);
        assert_ne!(next_key_1, next_key_2);
    }

    #[actix_web::main]
    #[test]
    async fn test_generate() {
        let sut = setup().await;
        let url = sut.generate("http://test.com").await.ok();
        assert!(matches!(url, Some(_url)));
    }

    #[actix_web::main]
    #[test]
    async fn test_generate_and_get() {
        let sut = setup().await;
        let unexists = sut.get("unexists").await.ok();
        assert_eq!(None, unexists);
        let url_1 = sut.generate("http://test.com").await.ok().unwrap();
        let url_2 = sut.get(&url_1.id).await.ok().unwrap();
        assert_eq!(url_2, url_1);
    }

    #[actix_web::main]
    #[test]
    async fn test_incr() {
        let sut = setup().await;
        let url_1 = sut.generate("http://test.com").await.unwrap();
        assert_eq!(url_1.count, 0);
        sut.increment_counter(&url_1.id).await.ok();
        let url_2 = sut.get(&url_1.id).await.unwrap();
        assert_eq!(url_2.count, 1);
    }

    #[actix_web::main]
    #[test]
    async fn test_new_user() {
        let sut = setup().await;
        let user_1 = sut.new_user().await.ok();
        let user_2 = sut.new_user().await.ok();
        assert_ne!(None, user_1);
        assert_ne!(user_1, user_2);
    }

    #[actix_web::main]
    #[test]
    async fn test_generate_for_user() {
        let sut = setup().await;
        let url = sut.generate_for_user("http://test.com", "user").await.ok();
        assert_ne!(None, url);
        assert!(matches!(url, Some(url) if url.url == "http://test.com"))
    }

    #[actix_web::main]
    #[test]
    async fn test_get_last_for_user() {
        let sut = setup().await;
        let url_1 = sut.generate_for_user("http://test.com", "user_new").await.unwrap();
        let url_2 = sut.generate_for_user("http://test.com", "user_new").await.unwrap();
        let res = sut.get_last_n_for_user("user_new", 2).await;
        assert_eq!(res, vec![url_2, url_1]);
    }

}
