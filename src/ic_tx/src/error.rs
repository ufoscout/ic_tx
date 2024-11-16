use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TxError {
    #[error("FetchError: {message}")]
    FetchError { message: String },
    #[error("FetchNotFoundError: {message}")]
    FetchNotFoundError { message: String },
    #[error("UpdateOptimisticLockError: {message}")]
    UpdateOptimisticLockError { message: String },
    #[error("UpdateError: {message}")]
    UpdateError { message: String },
    #[error("SaveError: {message}")]
    SaveError { message: String },
    #[error("DeleteError: {message}")]
    DeleteError { message: String },
    #[error("DeleteNotFoundError: {message}")]
    DeleteNotFoundError { message: String },
    #[error("DeleteOptimisticLockError: {message}")]
    DeleteOptimisticLockError { message: String },
}
