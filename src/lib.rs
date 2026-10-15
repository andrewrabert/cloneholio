pub mod cli;
pub mod config;
pub mod git;
pub mod github;
pub mod gitlab;
pub mod http;
pub mod log;
pub mod orphan;
pub mod repo;
pub mod run;
pub mod source;

pub const APP_NAME: &str = env!("CARGO_CRATE_NAME");
pub const APP_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), env!("VERSION_SUFFIX"));
