use std::fs::File;
use std::io::Cursor;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Instant;

use actix_multipart::Multipart;
use actix_web::web;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use futures_util::{StreamExt, TryStreamExt};
use serde_json::json;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;

use jcss::Predictor;

use crate::models::{Data, Message, Response, Status};

mod models;

#[get("/")]
async fn index() -> impl Responder {
    "JAccount Captcha Solver Service"
}

#[post("/")]
async fn predict(mut payload: Multipart, predictor: web::Data<Predictor>) -> impl Responder {
    let mut image: Option<Vec<u8>> = None;
    while let Some(item) = payload.next().await {
        if let Ok(field) = item {
            if field.name() == "image" {
                image = field
                    .try_fold(vec![], |mut acc, x| async move {
                        acc.extend(&*x);
                        Ok(acc)
                    })
                    .await
                    .ok();
            }
        }
    }

    let time_begin = Instant::now();

    let image = if let Some(image) = image {
        let image = image::io::Reader::new(Cursor::new(image))
            .with_guessed_format()
            .expect("seek never fail")
            .decode();
        match image {
            Ok(image) => image,
            Err(e) => {
                error!("Error occur when reading image - {}", e);
                return HttpResponse::BadRequest().json(Response::new(
                    Status::Fail,
                    Data::new(json!({"image": "Invalid image format."})),
                ));
            }
        }
    } else {
        return HttpResponse::BadRequest().json(Response::new(
            Status::Fail,
            Data::new(json!({"image": "A captcha image is required."})),
        ));
    };

    let result = web::block(move || predictor.predict(image))
        .await
        .expect("thread pool is present");
    let elapsed_time = time_begin.elapsed().as_millis();

    match result {
        Ok(s) => HttpResponse::Ok().json(Response::new(
            Status::Success,
            Data::new(json!({"prediction": s, "elapsed_time": elapsed_time.to_string()})),
        )),
        Err(e) => {
            error!("Error occur when predicting image - {}", e);
            HttpResponse::InternalServerError()
                .json(Response::new(Status::Error, Message::new(e.to_string())))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt().init();

    let bind = std::env::var("APP_BIND").expect("APP_BIND");
    let model = std::env::var("APP_MODEL").expect("APP_MODEL");

    info!("Loading inference model...");
    let predictor = web::Data::new(
        Predictor::new(File::open(model).expect("open model file"))
            .expect("failed to initialize model"),
    );

    info!("Spinning up server...");
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(predictor.clone())
            .service(index)
            .service(predict)
    })
    .bind(SocketAddr::from_str(bind.as_str()).expect("invalid bind address"))?
    .run()
    .await
}
