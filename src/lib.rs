pub mod alerts;
pub mod app;
pub mod handlers;
pub mod models;
pub mod planner;
pub mod ratelimit;
pub mod routes;
pub mod statement;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/flinderax_backend.rs"));
}
