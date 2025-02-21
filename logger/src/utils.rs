use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use std::fmt::Debug;
use tracing::*;
use tracing_futures::Instrument;

pub async fn request_with_span<Fut: std::future::Future<Output = Result<T, E>>, T, E: Debug>(fut: Fut) -> Result<T, E> {
    let req_id: String = rng().sample_iter(&Alphanumeric).take(10).map(char::from).collect();
    let req_id = req_id.as_str();
    let span = tracing::error_span!("req", req_id);

    with_span(span, async move {
        debug!("Start request [{}]", req_id);
        fut.await
            .map_err(|err| {
                error!("Request error: {:?}", err);
                err
            })
            .inspect(|_res| {
                debug!("Request completed successfully");
            })
    })
    .await
}

#[inline]
pub async fn with_span<Fut: std::future::Future>(span: Span, fut: Fut) -> Fut::Output {
    fut.instrument(span).await
}
