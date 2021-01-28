use super::types::*;
use async_trait::async_trait;
use redis::{AsyncCommands, RedisResult};
use std::sync::Arc;
use harsh::Harsh;
use std::time::{SystemTime, UNIX_EPOCH};

const URL_COUNTER_KEY: &str = "url_shortener:url_counter";
const USER_COUNTER_KEY: &str = "url_shortener:user_counter";
const URLS_KEY: &str = "url_shortener:urls";
const USERS_KEY: &str = "url_shortener:users";


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

    async fn get_next_key(self: &Self) -> Option<String> {
        let redis_client = &*self.redis_client;
        if let Ok(mut conn) = redis_client.get_async_connection().await {
            conn.incr(URL_COUNTER_KEY, 1)
                .await
                .map(|x|(self.hashids.encode(&[x,])))
                .ok()
        } else {
            None
        }
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
    async fn generate(&self, url: &str) -> Option<Url> {
        let redis_client = &*self.redis_client;
        let next_key = self.get_next_key().await.unwrap();
        let url_key = self.get_key(&next_key);
        if let Ok(mut conn) = redis_client.get_async_connection().await {
            conn.hset_multiple(
                url_key, &[("id", &*next_key), ("url", &*url), ("count", "0")]
            ).await.ok()?;
            Some(Url {id: next_key.clone(), url: url.parse().unwrap(), count: 0})
        } else {
            None
        }
    }

    async fn get(&self, id: &str) -> Option<Url> {
        let redis_client = &*self.redis_client;
        let url_key = self.get_key(&id);
        if let Ok(mut conn) = redis_client.get_async_connection().await {
            conn.hget(
                url_key, &["id", "url", "count"],
            ).await.map(
                |v: Vec<String>| {
                    if v.len() == 3 {
                        Some(Url {
                            id: v[0].clone(),
                            url: v[1].clone(),
                            count: v[2].parse().unwrap(),
                        })
                    } else {
                        None
                    }
                }).ok().unwrap()
        } else {
            None
        }
    }

    async fn increment_counter(&self, id: &str) -> bool {
        let redis_client = &*self.redis_client;
        let url_key = self.get_key(&id);
        if let Ok(mut conn) = redis_client.get_async_connection().await {
            conn.hincr(
                url_key, "count", 1
            ).await
                .map(|_: String| true)
                .unwrap_or(false)
        } else {
            false
        }
    }

    async fn new_user(&self) -> Option<String> {
        let redis_client = &*self.redis_client;
        if let Ok(mut conn) = redis_client.get_async_connection().await {
            conn.incr(USER_COUNTER_KEY, 1)
                .await
                .map(|x|(self.hashids.encode(&[x,])))
                .ok()
        } else {
            None
        }
    }

    async fn generate_for_user(&self, url: &str, user: &str) -> Option<Url> {
        let url = self.generate(url).await;
        match url {
            Some(url) => {
                self.save_for_user(&url.id.clone(), user).await;
                Some(url)
            },
            None => None
        }
    }

    async fn get_last_n_for_user(&self, user: &str, n:isize) -> Vec<Url> {
        let redis_client = &*self.redis_client;
        if let Ok(mut conn) = redis_client.get_async_connection().await {
            let ids:Vec<String> = conn.zrevrange(self.get_user_key(user), 0, n-1).await.unwrap_or(vec![]);
            let mut res = vec![];
            for key in ids.into_iter() {
                if let Some(url) = self.get(&*key).await {
                    res.push(url)
                }
            }
            res
        } else {
            vec![]
        }
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
        let next_key_1 = sut.get_next_key().await;
        let next_key_2 = sut.get_next_key().await;
        assert_ne!(None, next_key_1);
        assert_ne!(next_key_1, next_key_2);
    }

    #[actix_web::main]
    #[test]
    async fn test_generate() {
        let sut = setup().await;
        let url = sut.generate("http://test.com").await;
        assert_ne!(None, url);
        assert!(matches!(url, Some(url) if url.url == "http://test.com"))
    }

    #[actix_web::main]
    #[test]
    async fn test_generate_and_get() {
        let sut = setup().await;
        let unexists = sut.get("unexists").await;
        assert_eq!(None, unexists);
        let url_1 = sut.generate("http://test.com").await.unwrap();
        let url_2 = sut.get(&url_1.id).await.unwrap();
        assert_eq!(url_2, url_1);
    }

    #[actix_web::main]
    #[test]
    async fn test_incr() {
        let sut = setup().await;
        let url_1 = sut.generate("http://test.com").await.unwrap();
        assert_eq!(url_1.count, 0);
        sut.increment_counter(&url_1.id).await;
        let url_2 = sut.get(&url_1.id).await.unwrap();
        assert_eq!(url_2.count, 1);
    }

    #[actix_web::main]
    #[test]
    async fn test_new_user() {
        let sut = setup().await;
        let user_1 = sut.new_user().await;
        let user_2 = sut.new_user().await;
        assert_ne!(None, user_1);
        assert_ne!(user_1, user_2);
    }

    #[actix_web::main]
    #[test]
    async fn test_generate_for_user() {
        let sut = setup().await;
        let url = sut.generate_for_user("http://test.com", "user").await;
        assert_ne!(None, url);
        assert!(matches!(url, Some(url) if url.url == "http://test.com"))
    }

    #[actix_web::main]
    #[test]
    async fn test_get_last_for_user() {
        let sut = setup().await;
        let url_1 = sut.generate_for_user("http://test.com", "user").await.unwrap();
        let url_2 = sut.generate_for_user("http://test.com", "user").await.unwrap();
        let res = sut.get_last_n_for_user("user", 2).await;
        assert_eq!(res, vec![url_2, url_1]);
    }

}
