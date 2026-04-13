use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("DB_QUERY failed. {0}")]
    DbQueryFailure(String),

    #[error("{0}")]
    Other(String)
}

pub mod repositories;
pub mod ports;
pub mod node;

pub struct AppContext {
    pub health_port: Box<dyn ports::HealthPort>,
    pub node_repo: Box<dyn repositories::NodeRespository>,
    pub node_log_repo: Box<dyn repositories::NodeLogRespository>,
    pub node_cmd_repo: Box<dyn repositories::NodeCommandRepository>
}