#[allow(clippy::module_inception)]
pub mod agent;
pub mod registry;

pub mod proto {
    tonic::include_proto!("runner");
}

pub use agent::run_agent;
