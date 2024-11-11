// src/api/handlers.rs

use crate::api::api_epoch;


impl ResponseError for ApiError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::InternalServerError().body(self.error.to_string())
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

impl ServerNetworkResponse {
    /// Creates a success response for a server.
    pub fn success(server_id: [u8; 32], message_id: [u8; 32], timestamp: u64) -> Self {
        Self {
            server_id,
            response: NetworkResponse::Success { message_id },
            timestamp,
        }
    }

    /// Creates a failure response for a server.
    pub fn failure(
        server_id: [u8; 32],
        message_id: [u8; 32],
        error: String,
        timestamp: u64,
    ) -> Self {
        Self {
            server_id,
            response: NetworkResponse::Failure { message_id, error },
            timestamp,
        }
    }
}

impl ServerNetworkRequest {
    /// Creates a new server request for a given message type.
    pub fn new(server_id: [u8; 32], message: MessageType) -> Self {
        Self {
            server_id,
            message,
            timestamp: get_current_timestamp(),
        }
    }
}
