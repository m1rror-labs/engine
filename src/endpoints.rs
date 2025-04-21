use actix_multipart::Multipart;
use actix_web::{delete, get, post, put, rt, web, Error, HttpRequest, HttpResponse, Responder};
use actix_ws::AggregatedMessage;
use base64::prelude::*;
use futures::StreamExt as _;
use serde::Deserialize;
use solana_sdk::{account::Account, program_option::COption, program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;
use std::{env, str::FromStr, sync::Arc};

use serde_json::json;
use uuid::Uuid;

use crate::{
    engine::{builtins::BUILTINS, SvmEngine, SVM},
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
                    "message": e.to_string()
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

    BUILTINS
        .iter()
        .find(|builtin| builtin.program_id == program_id)
        .map(|_| {
            return HttpResponse::BadRequest().json(json!({
                "error": format!("Program id {} is a builtin program, and can't be overwritten", program_id)
            }));
        });

    let (pubkey, account) = svm.add_program(program_id, &program_data);
    match svm.storage.set_account(id, &pubkey, account, None) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Program loaded successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[derive(Deserialize)]
pub struct AccountReq {
    address: String,
    lamports: u64,
    data: String,
    owner: String,
    rent_epoch: u64,
    executable: bool,
    token_mint_auth: Option<String>,
}

#[put("/accounts/{id}")]
pub async fn load_account(
    accounts_req: web::Json<Vec<AccountReq>>,
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    let accounts = accounts_req.iter().map(|account| {
        let mut data = match BASE64_STANDARD.decode(&account.data) {
            Ok(data) => data,
            Err(_) => {
                return Err("Invalid base64 data".to_string());
            }
        };

        if account.token_mint_auth.is_some() {
            let token_mint_signer =
                match Pubkey::from_str(&account.token_mint_auth.as_ref().unwrap()) {
                    Ok(token_mint_signer) => token_mint_signer,
                    Err(_) => {
                        return Err("Invalid token mint signer".to_string());
                    }
                };
            let mut mint_data = match Mint::unpack(&data) {
                Ok(mint_data) => mint_data,
                Err(_) => {
                    return Err("Invalid mint data".to_string());
                }
            };
            mint_data.mint_authority = COption::Some(token_mint_signer);
            match Mint::pack(mint_data, &mut data) {
                Ok(data) => data,
                Err(_) => {
                    return Err("Invalid mint data".to_string());
                }
            };
        }

        let owner = match Pubkey::from_str(&account.owner) {
            Ok(owner) => owner,
            Err(_) => {
                return Err("Invalid owner".to_string());
            }
        };
        let address = match Pubkey::from_str(&account.address) {
            Ok(address) => address,
            Err(_) => {
                return Err("Invalid address".to_string());
            }
        };
        Ok((
            address,
            Account {
                lamports: account.lamports,
                data: data,
                owner: owner,
                rent_epoch: account.rent_epoch,
                executable: account.executable,
            },
        ))
    });

    let accounts: Vec<(Pubkey, Account)> = match accounts.collect() {
        Ok(accounts) => accounts,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": e
            }));
        }
    };

    match svm.storage.set_accounts(id, accounts) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Account loaded successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateBlockchainReq {
    pub config: Option<Uuid>,
    pub defer_account_initailization: Option<bool>,
}

#[post("/blockchains")]
pub async fn create_blockchain(
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    http_req: HttpRequest,
    req: Option<web::Json<CreateBlockchainReq>>,
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
        let user_id = match http_req.headers().get("user_id") {
            Some(user_id) => match user_id.to_str() {
                Ok(user_id) => user_id.to_string(),
                Err(_) => {
                    return HttpResponse::BadRequest().json(json!({
                        "message": "Invalid user_id header"
                    }))
                }
            },
            None => {
                return HttpResponse::BadRequest().json(json!({
                    "message": "Missing user_id header"
                }))
            }
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
    let config = match &req {
        Some(req) => req.config,
        None => None,
    };
    let id = svm.create_blockchain(team.id, None, label, expiry, config);
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

#[derive(Deserialize, Debug, Clone)]
pub struct ConvertAccountToConfigReq {
    pub account: String,
    pub blockchain: Uuid,
    pub config: Uuid,
}

#[post("/accounts/convert")]
pub async fn convert_account_to_config(
    svm: web::Data<Arc<SvmEngine<PgStorage>>>,
    req: web::Json<ConvertAccountToConfigReq>,
) -> impl Responder {
    let pubkey = match Pubkey::from_str(&req.account) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Invalid account address"
            }));
        }
    };

    let account = match svm.storage.get_account(req.blockchain, &pubkey, false) {
        Ok(account) => match account {
            Some(account) => account,
            None => {
                return HttpResponse::BadRequest().json(json!({
                    "message": "Account not found"
                }));
            }
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(e.to_string());
        }
    };

    // let mut mint = Mint::unpack(&account.data).unwrap();
    // println!("{:?}", mint.mint_authority);
    // let authority = pubkey!("5YNmS1R9nNSCDzb5a7mMJ1dwK9uHeAAF4CmPEwKgVWr8");
    // mint.mint_authority = COption::Some(authority);

    // mint.pack_into_slice(&mut account.data);

    match svm.storage.set_config_account(req.config, &pubkey, account) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Account converted to config account successfully"
        })),
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
