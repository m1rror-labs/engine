use actix_cors::Cors;
use actix_web::{middleware, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger::Env;
use litesvm::LiteSVM;
use mockchain_engine::rpc::rpc::{handle_request, Dependencies, RpcRequest};

#[post("/")]
async fn rpc_reqest(req: web::Json<RpcRequest>, deps: web::Data<Dependencies>) -> impl Responder {
    let res = handle_request(req.into_inner(), &deps);
    HttpResponse::Ok().json(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let svm = LiteSVM::new();

    let deps = Dependencies::new(svm);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(deps.clone())) // Share dependencies
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
