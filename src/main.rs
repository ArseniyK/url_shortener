use std::sync::Arc;

use actix_files;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{web, App, HttpServer};
use harsh::Harsh;
use tera::Tera;

mod hashids;
mod redis;
mod urls;

fn configure(redis_client: Arc<redis::Client>, hashids: Harsh, cfg: &mut web::ServiceConfig) {
    use crate::urls::api;
    use crate::urls::redis_url_repo::RedisUrlRepoImpl;
    use crate::urls::url_service::UrlServiceImpl;
    let url_repo = RedisUrlRepoImpl {
        redis_client,
        hashids,
    };
    let url_service = UrlServiceImpl { url_repo };
    api::configure(web::Data::new(url_service), cfg);
}

#[actix_web::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    let redis_client = Arc::new(redis::configure().await);
    let hashids = hashids::configure().await;
    let template = Tera::new("templates/**/*").unwrap();

    let port = std::env::var("PORT").expect("PORT env var must be set");
    let bind = format!("0.0.0.0:{}", port);

    let secret_key = std::env::var("SECRET").expect("SECRET env var must be set");

    let app = move || {
        App::new()
            .data(template.clone())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&secret_key.as_bytes())
                    .name("auth")
                    .secure(false)
                    .max_age(315576000), // 10 years
            ))
            .service(actix_files::Files::new("/static/", "static/").use_last_modified(true))
            .configure(|cfg| configure(redis_client.clone(), hashids.clone(), cfg))
    };

    HttpServer::new(app)
        .bind(bind)
        .expect("Unable to bind server")
        .run()
        .await
        .expect("Failed to start web server")
}
