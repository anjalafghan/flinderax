pub mod alerts;
pub mod app;
pub mod backup;
pub mod handlers;
pub mod headers;
pub mod joy;
pub mod import_joy;
pub mod models;
pub mod narration;
pub mod planner;
pub mod ratelimit;
pub mod routes;
pub mod statement;
pub mod validate;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/joy_backend.rs"));
}
