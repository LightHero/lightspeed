use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tracing::*;
use tracing_futures::Instrument;

pub async fn request_with_span<Fut: std::future::Future<Output = Result<T, E>>, T, E: std::fmt::Display>(
    fut: Fut,
) -> Result<T, E> {
    let req_id: String = thread_rng().sample_iter(&Alphanumeric).take(10).map(char::from).collect();
    let req_id = req_id.as_str();
    let span = tracing::error_span!("req", req_id);

    async move {
        debug!("Start request [{}]", req_id);
        fut.await
            .map_err(|err| {
                error!("Request error: {}", err);
                err
            })
            .map(|res| {
                debug!("Request completed successfully");
                res
            })
    }
    .instrument(span)
    .await
}
