use actix_multipart::Multipart;
use actix_web::{delete, get, post, rt, web, Error, HttpRequest, HttpResponse, Responder};
use actix_ws::AggregatedMessage;
use futures::StreamExt as _;
use std::sync::Arc;

use serde_json::json;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    rpc::{
        rpc::{handle_request, RpcRequest},
        ws::handle_ws_request,
    },
    storage::PgStorage,
};

pub async fn rpc_reqest(
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

pub async fn rpc_ws(
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

#[post("/programs/{id}")]
pub async fn load_program(
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
pub async fn create_blockchain(svm: web::Data<Arc<SvmEngine<PgStorage>>>) -> impl Responder {
    let id = svm.create_blockchain(None);
    match id {
        Ok(id) => HttpResponse::Ok().json(json!({
            "url": format!("https://rpc.mockchain.app/rpc/{}", id.to_string())
        })),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[get("/blockchains")]
pub async fn get_blockchains(svm: web::Data<Arc<SvmEngine<PgStorage>>>) -> impl Responder {
    let res = svm.get_blockchains();
    match res {
        Ok(blockchains) => HttpResponse::Ok().json(json!({
            "blockchains": blockchains.iter().map(|b| format!("https://rpc.mockchain.app/rpc/{}", b.id.to_string())).collect::<Vec<String>>()
        })),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[delete("/blockchains/{id}")]
pub async fn delete_blockchain(
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
