use thiserror::Error;
use tract_onnx::prelude::TractError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("tract error - {0}")]
    Tract(#[from] TractError),
}
