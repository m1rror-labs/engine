use actix_multipart::Multipart;
use actix_web::{delete, get, post, put, rt, web, Error, HttpRequest, HttpResponse, Responder};
use actix_ws::AggregatedMessage;
use futures::StreamExt as _;
use serde::Deserialize;
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::{env, sync::Arc};

use serde_json::json;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    rpc::{
        rpc::{handle_request, RpcMethod, RpcRequest},
        ws::handle_ws_request,
    },
    storage::{teams::Team, PgStorage, Storage},
};

pub async fn rpc_reqest(
    req: web::Json<RpcRequest>,
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    let res = handle_request(id, req.clone(), &svm);
    println!("{:?}", req.method);
    if req.method != RpcMethod::GetAccountInfo {
        println!("{:?}", res);
    }
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
                            match session
                                .text(
                                    serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "id": serde_json::Value::Null,
                                        "error": {
                                            "code": -32603,
                                            "message": e
                                        }
                                    })
                                    .to_string(),
                                )
                                .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("{:?}", e);
                                }
                            }
                        }
                    }
                }
                Ok(AggregatedMessage::Binary(bin)) => match session.binary(bin).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!("{:?}", e);
                    }
                },
                Ok(AggregatedMessage::Ping(msg)) => match session.pong(&msg).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!("{:?}", e);
                    }
                },
                Ok(AggregatedMessage::Close(reason)) => {
                    println!("Client disconnected: {:?}", reason);
                    match session.close(reason).await {
                        Ok(_) => {}
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    };
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
        let mut field = match item {
            Ok(i) => i,
            Err(e) => {
                return HttpResponse::BadRequest().json(json!({
                    "error": e.to_string()
                }));
            }
        };
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

#[derive(Deserialize)]
pub struct AccountReq {
    address: Pubkey,
    lamports: u64,
    data: Vec<u8>,
    owner: Pubkey,
    rent_epoch: u64,
    label: Option<String>,
}

#[put("/accounts/{id}")]
pub async fn load_account(
    account: web::Json<AccountReq>,
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    let acc = Account {
        lamports: account.lamports,
        data: account.data.clone(),
        owner: account.owner,
        executable: false, //Must go through upload program to upload executable accounts
        rent_epoch: account.rent_epoch,
    };
    match svm
        .storage
        .set_account(id, &account.address, acc, account.label.clone())
    {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Account loaded successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[post("/blockchains")]
pub async fn create_blockchain(
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    http_req: HttpRequest,
) -> impl Responder {
    let team = match get_team(svm.clone(), http_req.clone()) {
        Ok(team_id) => team_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(json!({
                "message": e
            }))
        }
    };

    let existing_blockchains = match svm.get_blockchains(team.id) {
        Ok(blockchains) => blockchains,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    if team.default_expiry == None && existing_blockchains.len() >= 10 {
        return HttpResponse::BadRequest().json(json!({
            "message": "You can only create 10 blockchains per team"
        }));
    }

    let mut label = None;
    if team.default_expiry.is_some() {
        let user_id = match http_req.headers().get("user_id"){
            Some(user_id) => match user_id.to_str() {
                Ok(user_id) => user_id.to_string(),
                Err(_) => return HttpResponse::BadRequest().json(json!({
                    "message": "Invalid user_id header"
                })),
            },
            None => return HttpResponse::BadRequest().json(json!({
                "message": "Missing user_id header"
            })),
        };
        if user_id == "" {
            return HttpResponse::BadRequest().json(json!({
                "message": "user_id header cannot be empty"
            }));
        }
        label = Some(user_id);
    }

    let expiry = match team.default_expiry {
        Some(expiry) => {
            Some(chrono::Utc::now().naive_utc() + chrono::Duration::seconds(expiry as i64))
        }
        None => None,
    };
    let id = svm.create_blockchain(team.id, None,label, expiry);
    match id {
        Ok(id) => {
            let mut base_url = "https://rpc.mirror.ad/rpc/";
            if env::var("ENV").unwrap_or_else(|_| "prod".to_string()) == "dev" {
                base_url = "http://localhost:8899/rpc/";
            }
            HttpResponse::Ok().json(json!({
                "url": format!("{}{}",base_url, id.to_string())
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[post("/blockchains/expire")]
pub async fn expire_blockchains(svm: web::Data<Arc<SvmEngine<PgStorage>>>) -> impl Responder {
    let expired_blockchains = match svm.storage.get_expired_blockchains() {
        Ok(blockchains) => blockchains,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    for blockchain in expired_blockchains {
        if let Err(e) = svm.delete_blockchain(blockchain.id) {
            println!("Error deleting blockchain {}: {}", blockchain.id, e);
        }
    }

    HttpResponse::Ok().json(json!({
        "message": "Expired blockchains deleted successfully"
    }))
}

#[get("/blockchains")]
pub async fn get_blockchains(
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    http_req: HttpRequest,
) -> impl Responder {
    let team_id = match get_team_id(svm.clone(), http_req) {
        Ok(team_id) => team_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(json!({
                "message": e
            }))
        }
    };
    let res = svm.get_blockchains(team_id);
    match res {
        Ok(blockchains) => HttpResponse::Ok().json(json!({
            "blockchains": blockchains.iter().map(|b| format!("https://rpc.mirror.ad/rpc/{}", b.id.to_string())).collect::<Vec<String>>()
        })),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[delete("/blockchains")]
pub async fn delete_blockchains(
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    http_req: HttpRequest,
) -> impl Responder {
    let team_id = match get_team_id(svm.clone(), http_req) {
        Ok(team_id) => team_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(json!({
                "message": e
            }))
        }
    };
    let blockchains = match svm.get_blockchains(team_id) {
        Ok(blockchains) => blockchains,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    for blockchain in blockchains {
        svm.delete_blockchain(blockchain.id).unwrap();
    }

    HttpResponse::Ok().json(json!({
        "message": "All blockchains deleted successfully"
    }))
}

pub async fn delete_blockchain(
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    path: web::Path<Uuid>,
    http_req: HttpRequest,
) -> impl Responder {
    let id = path.into_inner();
    let blockchain = match svm.storage.get_blockchain(id) {
        Ok(blockchain) => blockchain,
        Err(e) => {
            return HttpResponse::InternalServerError().json(e.to_string());
        }
    };
    if !valid_api_key(blockchain.team_id, svm.clone(), http_req) {
        return HttpResponse::Unauthorized().json(json!({
            "message": "Invalid API key"
        }));
    }
    let res = svm.delete_blockchain(id);
    match res {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Blockchain deleted successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

fn valid_api_key(
    id: Uuid,
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    http_req: HttpRequest,
) -> bool {
    let api_key = http_req
        .headers()
        .get("api_key")
        .and_then(|header_value| header_value.to_str().ok())
        .unwrap_or("");
    let api_key = match Uuid::parse_str(api_key) {
        Ok(api_key) => api_key,
        Err(_) => {
            return false;
        }
    };
    let team = match svm.storage.get_team_from_api_key(api_key) {
        Ok(team) => team,
        Err(_) => {
            return false;
        }
    };
    if team.id != id {
        return false;
    }
    true
}

fn get_team_id(
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    http_req: HttpRequest,
) -> Result<Uuid, String> {
    let api_key = http_req
        .headers()
        .get("api_key")
        .and_then(|header_value| header_value.to_str().ok())
        .unwrap_or("");

    match Uuid::parse_str(api_key) {
        Ok(api_key) => match svm.storage.get_team_from_api_key(api_key) {
            Ok(team) => Ok(team.id),
            Err(_) => Err("Invalid API key".to_string()),
        },
        Err(_) => Err("Invalid API key".to_string()),
    }
}

fn get_team(
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    http_req: HttpRequest,
) -> Result<Team, String> {
    let api_key = http_req
        .headers()
        .get("api_key")
        .and_then(|header_value| header_value.to_str().ok())
        .unwrap_or("");

    match Uuid::parse_str(api_key) {
        Ok(api_key) => match svm.storage.get_team_from_api_key(api_key) {
            Ok(team) => Ok(team),
            Err(_) => Err("Invalid API key".to_string()),
        },
        Err(_) => Err("Invalid API key".to_string()),
    }
}
