// src/api/routes.rs

use crate::api::handlers;
use crate::api::stats;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/epochs/{id}", web::get().to(handlers::get_epoch));
    cfg.route(
        "/epochs/recent/{limit}",
        web::get().to(handlers::get_recent_epochs),
    );
    cfg.route(
        "/epochs/{id}/transactions",
        web::get().to(handlers::get_epoch_transactions),
    );
    cfg.route("/stats", web::get().to(stats::get_chain_stats));
    cfg.route("/stats/epochs", web::get().to(stats::get_chain_stats));
    cfg.route("/stats/transactions", web::get().to(stats::get_chain_stats));
}
