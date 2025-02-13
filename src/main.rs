use std::{env, sync::Arc};

use actix_cors::Cors;
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use mockchain_engine::{
    engine::{SvmEngine, SVM},
    rpc::rpc::{handle_request, RpcRequest},
    storage::{self, PgStorage},
};

use serde_json::json;
use uuid::Uuid;

#[post("/rpc/{id}")]
async fn rpc_reqest(
    req: web::Json<RpcRequest>,
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let res = handle_request(id, req.clone(), &svm);
    req.params.as_ref().map(|p| println!("{:?}", p));
    println!("{}: {:?}", req.method, res);
    HttpResponse::Ok().json(res)
}

#[post("/blockchains")]
async fn create_blockchain(svm: web::Data<Arc<SvmEngine<PgStorage>>>) -> impl Responder {
    let id = svm.create_blockchain(None);
    match id {
        Ok(id) => HttpResponse::Ok().json(json!({
            "url": format!("https://rpc.mockchain.app/rpc/{}", id.to_string())
        })),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[get("/blockchains")]
async fn get_blockchains(svm: web::Data<Arc<SvmEngine<PgStorage>>>) -> impl Responder {
    let res = svm.get_blockchains();
    match res {
        Ok(blockchains) => HttpResponse::Ok().json(json!({
            "blockchains": blockchains.iter().map(|b| format!("https://rpc.mockchain.app/rpc/{}", b.id.to_string())).collect::<Vec<String>>()
        })),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

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
            .service(create_blockchain)
            .service(get_blockchains)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
