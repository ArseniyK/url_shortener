use actix_web::{error, http, web, Error, HttpResponse, Result};
use validator::Validate;

use super::types::*;
use actix_identity::Identity;
use tera::Tera;

pub fn configure<T: 'static + UrlService>(service: web::Data<T>, cfg: &mut web::ServiceConfig) {
    cfg.app_data(service);
    cfg.route("/", web::get().to(index::<T>));
    cfg.route("/", web::post().to(shorten::<T>));
    cfg.route("/{id}", web::get().to(redirect::<T>));
}

fn serialize(url: Url) -> ResponseUrl {
    ResponseUrl {
        id: url.id.clone(),
        short_url: build_url(url.id),
        long_url: url.url.clone(),
        count: url.count.clone(),
    }
}

fn build_url(id: String) -> String {
    let base_url = std::env::var("DOMAIN").expect("DOMAIN");
    let schema: &str = if base_url.starts_with("localhost") {
        "http://"
    } else {
        "https://"
    };
    format!("{}{}/{}", schema, base_url, id)
}

pub async fn index<T: UrlService>(
    service: web::Data<T>,
    identity: Identity,
    template: web::Data<Tera>,
) -> Result<HttpResponse, Error> {
    let user = identity
        .identity()
        .unwrap_or(service.new_user().await.unwrap());
    identity.remember(user.clone());
    let mut ctx = tera::Context::new();
    let lasts: Vec<ResponseUrl> = service
        .get_last_n_for_user(&*user, 25)
        .await
        .into_iter()
        .map(|u: Url| serialize(u))
        .collect();
    ctx.insert("lasts", &lasts);
    let res = template
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().body(res))
}

async fn shorten<T: UrlService>(
    service: web::Data<T>,
    identity: Identity,
    data: web::Json<CreateUrl>,
) -> Result<HttpResponse, Error> {
    let user = identity
        .identity()
        .unwrap_or(service.new_user().await.unwrap());
    identity.remember(user.clone());

    let url_create = data.into_inner();
    url_create
        .validate()
        .map_err(|e| error::ErrorBadRequest(serde_json::to_string(&e).unwrap()))?;

    let result = service.shorten(&url_create.url, &user).await;
    match result {
        Ok(url) => Ok(HttpResponse::Ok().json(serialize(url))),
        _ => Ok(HttpResponse::BadRequest().finish()),
    }
}

pub async fn redirect<T: UrlService>(
    service: web::Data<T>,
    params: web::Path<RedirectParams>,
) -> Result<HttpResponse, Error> {
    let result = service.get(&params.id).await;
    match result {
        Ok(result) => Ok(HttpResponse::MovedPermanently()
            .header(http::header::LOCATION, result.url)
            .finish()),
        _ => Ok(HttpResponse::BadRequest().finish()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use mockall::predicate::*;

    #[actix_web::main]
    #[test]
    async fn test_shorten_wrong() {
        std::env::set_var("DOMAIN", "localhost");
        let url = Url {
            id: "test".to_string(),
            url: "http://test.com".to_string(),
            count: 0,
        };

        let input = CreateUrl {
            url: "test".to_string(),
        };

        let mut url_service = MockUrlService::new();
        url_service
            .expect_shorten()
            .with(eq("test"), eq("user"))
            .times(0)
            .return_const(Ok(url));
        url_service
            .expect_new_user()
            .return_const(Ok("user".to_string()));
        let url_service = web::Data::new(url_service);

        let mut sut =
            test::init_service(App::new().configure(|cfg| configure(url_service, cfg))).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&input)
            .to_request();
        let resp = test::call_service(&mut sut, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        println!(rest.body())
    }

    #[actix_web::main]
    #[test]
    async fn test_shorten_correct() {
        std::env::set_var("DOMAIN", "localhost");
        let url = Url {
            id: "test".to_string(),
            url: "http://test.com".to_string(),
            count: 0,
        };

        let mut url_service = MockUrlService::new();
        url_service
            .expect_shorten()
            .with(eq("http://test.com"), eq("user"))
            .times(1)
            .return_const(Ok(url.clone()));
        url_service
            .expect_new_user()
            .return_const(Ok("user".to_string()));
        let url_service = web::Data::new(url_service);

        let mut sut =
            test::init_service(App::new().configure(|cfg| configure(url_service, cfg))).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&CreateUrl {
                url: "http://test.com".to_string(),
            })
            .to_request();
        let resp = test::call_service(&mut sut, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::main]
    #[test]
    async fn test_redirect() {
        std::env::set_var("DOMAIN", "localhost");
        let url = Url {
            id: "test".to_string(),
            url: "http://test.com".to_string(),
            count: 0,
        };

        let mut url_service = MockUrlService::new();
        url_service.expect_get().return_const(Ok(url.clone()));
        let url_service = web::Data::new(url_service);

        let mut sut =
            test::init_service(App::new().configure(|cfg| configure(url_service, cfg))).await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&mut sut, req).await;
        assert_eq!(resp.status(), StatusCode::MOVED_PERMANENTLY);
    }
}
