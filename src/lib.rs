#![allow(clippy::inconsistent_digit_grouping)] // amounts are written in lakh grouping (12_34_567)
// Style lints that would force churn on stable code: wide SQL helpers take many columns, and the
// handlers return `Response` as the error. Revisit during the structure phase (findings G1/D4).
#![allow(clippy::too_many_arguments, clippy::type_complexity, clippy::result_large_err)]
pub mod alerts;
pub mod app;
pub mod backup;
pub mod handlers;
pub mod headers;
pub mod joy;
pub mod models;
pub mod narration;
pub mod planner;
pub mod ratelimit;
pub mod routes;
pub mod statement;
pub mod validate;

