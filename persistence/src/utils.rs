use lightspeed_core::error::{ErrorCodes, LsError};
use std::path::Path;

pub async fn read_file<W: tokio::io::AsyncWrite + Unpin + Send>(
    file_path: impl AsRef<Path>,
    output: &mut W,
) -> Result<u64, LsError> {
    let file_path_ref = file_path.as_ref();
    let mut file = tokio::fs::File::open(file_path_ref).await.map_err(|err| LsError::BadRequest {
        message: format!("Cannot open file [{}]. Err: {:?}", file_path_ref.display(), err),
        code: ErrorCodes::IO_ERROR,
    })?;
    tokio::io::copy(&mut file, output).await.map_err(|err| LsError::BadRequest {
        message: format!("Cannot copy file content to output writer [{}]. Err: {:?}", file_path_ref.display(), err),
        code: ErrorCodes::IO_ERROR,
    })
}
