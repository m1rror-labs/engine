use std::{env, sync::Arc};

use actix_cors::Cors;
use actix_web::{middleware, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger::Env;
use mockchain_engine::{
    engine::{SvmEngine, SVM},
    rpc::rpc::{handle_request, RpcRequest},
    storage::{self, PgStorage},
};

use uuid::Uuid;

#[post("/{id}")]
async fn rpc_reqest(
    req: web::Json<RpcRequest>,
    svm: web::Data<SvmEngine<PgStorage>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let res = handle_request(id, req.into_inner(), &svm);
    HttpResponse::Ok().json(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let storage = storage::PgStorage::new(&database_url);
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
            .service(rpc_reqest)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
