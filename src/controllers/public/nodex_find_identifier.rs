use serde::{Deserialize, Serialize};
use actix_web::{ HttpRequest, HttpResponse, web };

// NOTE: GET /identifiers/${ did }
#[derive(Deserialize, Serialize)]
struct MessageContainer {}

pub async fn handler(
    _req: HttpRequest,
    did: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let service = crate::services::nodex::NodeX::new();

    match service.find_identifier(&did).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().json(&v))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}