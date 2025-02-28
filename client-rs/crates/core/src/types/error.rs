use thiserror::Error;

use crate::{
    clients::ClientError, db::manager::DatabaseError, types::{contract::ContractError, proofs::ProofError},
    utils::UtilsError,
};

#[derive(Debug, Error)]
pub enum BankaiCoreError {
    #[error("Proof error: {0}")]
    Proof(#[from] ProofError),
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Client error: {0}")]
    Client(#[from] ClientError),
    #[error("Contract error: {0}")]
    Contract(#[from] ContractError),
    #[error("Utils error: {0}")]
    Utils(#[from] UtilsError),
}