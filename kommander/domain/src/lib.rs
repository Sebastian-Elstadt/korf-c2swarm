use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("DB_QUERY failed. {0}")]
    DbQueryFailure(String),

    #[error("{0}")]
    Other(String)
}

pub mod repositories;
pub mod node;

pub struct AppContext {
    pub node_repo: Box<dyn repositories::NodeRespository>
}