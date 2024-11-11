// src/api/middleware.rs


impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().body(self.error.to_string())
    }
}

// trait response error
pub trait ResponseError {
    fn error_response(&self) -> HttpResponse;
}

/// Handler for fetching a specific epoch.
pub async fn get_epoch(epoch_id: web::Path<u64>) -> HttpResponse {
    match api_epoch::get_epoch(*epoch_id) {
        Ok(epoch_data) => HttpResponse::Ok().json(epoch_data),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Handler for fetching recent epochs.
pub async fn get_recent_epochs(limit: web::Path<u32>) -> HttpResponse {
    match api_epoch::get_recent_epochs(*limit) {
        Ok(epochs) => HttpResponse::Ok().json(epochs),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Handler for fetching transactions in an epoch.
pub async fn get_epoch_transactions(epoch_id: web::Path<u64>) -> HttpResponse {
    match api_epoch::get_epoch_transactions(*epoch_id) {
        Ok(transactions) => HttpResponse::Ok().json(transactions),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
