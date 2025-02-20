use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{
    delete, get, middleware, post, rt, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use actix_ws::AggregatedMessage;
use dotenv::dotenv;
use futures::StreamExt as _;
use mockchain_engine::{
    engine::{SvmEngine, SVM},
    rpc::{
        rpc::{handle_request, RpcRequest},
        ws::handle_ws_request,
    },
    storage::{self, PgStorage},
};
use std::{env, sync::Arc};

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
    println!("{:?}", req.method);
    println!("{:?}", res);
    HttpResponse::Ok().json(res)
}

#[post("/programs/{id}")]
async fn load_program(
    mut payload: Multipart,
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let mut program_data = Vec::new();
    let mut program_id_str = String::new();

    // Parse the file from the request
    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();
        if field.name() == Some("program") {
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                program_data.extend_from_slice(&data);
            }
        }
        if field.name() == Some("program_id") {
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                program_id_str.push_str(&String::from_utf8_lossy(&data));
            }
        }
    }

    let program_id = match program_id_str.parse() {
        Ok(program_id) => program_id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
              "message": "Invalid program id"
            }));
        }
    };

    match svm.add_program(id, program_id, &program_data) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Program loaded successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
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

#[delete("/blockchains/{id}")]
async fn delete_blockchain(
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let res = svm.delete_blockchain(id);
    match res {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Blockchain deleted successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn rpc_ws(
    req: HttpRequest,
    path: web::Path<Uuid>,
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));
    let id = path.into_inner();
    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    println!("{:?}", text);
                    let res = handle_ws_request(id, &text.to_string(), session.clone(), &svm).await;
                    match res {
                        Ok(_) => {}
                        Err(e) => {
                            session.text(e).await.unwrap();
                        }
                    }
                }
                Ok(AggregatedMessage::Binary(bin)) => {
                    session.binary(bin).await.unwrap();
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }
                Ok(AggregatedMessage::Close(reason)) => {
                    println!("Client disconnected: {:?}", reason);
                    session.close(reason).await.unwrap();
                    break;
                }
                _ => {}
            }
        }
    });
    Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let storage = storage::PgStorage::new(&database_url);
    let svm = Arc::new(SvmEngine::new(storage.clone()));

    if env::var("ENV").unwrap_or_else(|_| "prod".to_string()) == "dev" {
        rt::spawn(async move {
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
                    .route("/rpc/{id}", web::get().to(rpc_ws))
            })
            .bind(("0.0.0.0", 8081))?
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
            .service(web::resource("/rpc/{id}").route(web::get().to(rpc_ws))) // WebSocket route
            .service(rpc_reqest)
            .service(create_blockchain)
            .service(get_blockchains)
            .service(delete_blockchain)
            .service(load_program)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
