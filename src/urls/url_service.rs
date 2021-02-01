use super::types::*;
use crate::urls::error::UrlError;
use async_trait::async_trait;

pub struct UrlServiceImpl<A: UrlRepo> {
    pub url_repo: A,
}

#[async_trait]
impl<A> UrlService for UrlServiceImpl<A>
where
    A: UrlRepo + Sync + Send,
{
    async fn shorten(&self, url: &str, user: &str) -> Result<Url, UrlError> {
        self.url_repo.generate_for_user(url, user).await
    }

    async fn get(&self, id: &str) -> Result<Url, UrlError> {
        let url = self.url_repo.get(id).await;
        match url {
            Ok(url) => {
                self.url_repo.increment_counter(id).await.ok();
                Ok(url)
            }
            Err(e) => Err(e),
        }
    }

    async fn new_user(&self) -> Result<String, UrlError> {
        self.url_repo.new_user().await
    }

    async fn get_last_n_for_user(&self, user: &str, n: isize) -> Vec<Url> {
        self.url_repo.get_last_n_for_user(user, n).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[actix_web::main]
    #[test]
    async fn test_shorten_and_get_success() {
        let id = "test";
        let long_url = "http://test.com";
        let user = "user";

        let url = Url {
            id: id.to_string(),
            url: long_url.to_string(),
            count: 0,
        };

        let mut url_repo = MockUrlRepo::new();
        url_repo
            .expect_generate_for_user()
            .return_const(Ok(url.clone()));
        url_repo
            .expect_get()
            .with(eq(id.clone()))
            .return_const(Ok(url.clone()));
        url_repo
            .expect_increment_counter()
            .times(1)
            .return_const(Ok(true));

        let sut = UrlServiceImpl { url_repo };

        let result = sut.shorten(&long_url, &user).await.ok();
        let expected = Some(url.clone());
        assert_eq!(expected, result);

        let result = sut.get(&id).await.ok();
        let expected = Some(url.clone());
        assert_eq!(expected, result);
    }
}
