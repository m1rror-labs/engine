use actix_cors::Cors;
use actix_web::{middleware, rt, web, App, HttpServer};

use dotenv::dotenv;

use mockchain_engine::{
    endpoints::{
        create_blockchain, delete_blockchain, delete_blockchains, expire_blockchains,
        get_blockchains, load_account, load_program, rpc_reqest, rpc_ws,
    },
    engine::{SvmEngine, SVM},
    storage::{self},
};
use std::{env, sync::Arc};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let cache_url = env::var("CACHE_URL").expect("CACHE_URL must be set");
    let storage = storage::PgStorage::new(&database_url, &cache_url);
    let svm = Arc::new(SvmEngine::new(storage.clone()));

    if env::var("ENV").unwrap_or_else(|_| "prod".to_string()) == "dev" {
        rt::spawn(async move {
            let storage = storage::PgStorage::new(&database_url, &cache_url);
            let svm = Arc::new(SvmEngine::new(storage.clone()));
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(svm.clone())) // Share dependencies
                    .wrap(middleware::Logger::default())
                    .wrap(
                        Cors::default()
                            .allow_any_origin()
                            .allow_any_method()
                            .allow_any_header()
                            .supports_credentials(),
                    )
                    .route("/rpc/{id}", web::get().to(rpc_ws))
            })
            .bind(("0.0.0.0", 8900))?
            .run()
            .await
        });
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(svm.clone())) // Share dependencies
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials(),
            )
            .service(
                web::resource("/rpc/{id}")
                    .route(web::get().to(rpc_ws))
                    .route(web::delete().to(delete_blockchain))
                    .route(web::post().to(rpc_reqest)),
            )
            .service(create_blockchain)
            .service(get_blockchains)
            .service(expire_blockchains)
            .service(load_program)
            .service(delete_blockchains)
            .service(load_program)
            .service(load_account)
    })
    .bind(("0.0.0.0", 8899))?
    .bind(("::", 9001))?
    .run()
    .await
}
