use lightspeed_core::error::LightSpeedError;

pub async fn read_file<W: tokio::io::AsyncWrite + Unpin + Send>(file_path: &str, output: &mut W) -> Result<u64, LightSpeedError> {

    use tokio::io::AsyncReadExt;

    let mut file = tokio::fs::File::open(file_path)
        .await
        .map_err(|err| LightSpeedError::BadRequest {
            message: format!(
                "Cannot open file [{}]. Err: {}",
                file_path,
                err
            ),
        })?;

    tokio::io::copy(&mut file, output).await.map_err(|err| LightSpeedError::BadRequest {
        message: format!(
            "Cannot copy file content to output writer [{}]. Err: {}",
            file_path,
            err
        ),
    })
}