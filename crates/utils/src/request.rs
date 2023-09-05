use std::future::Future;

#[tracing::instrument(skip_all)]
pub async fn retry<F, Fut, T>(f: F) -> Result<T, reqwest_middleware::Error>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, reqwest_middleware::Error>>,
{
    retry_custom(|| async { Ok((f)().await) }).await
}

#[tracing::instrument(skip_all)]
async fn retry_custom<F, Fut, T>(f: F) -> Result<T, reqwest_middleware::Error>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Result<T, reqwest_middleware::Error>, reqwest_middleware::Error>>,
{
    let mut response: Option<Result<T, reqwest_middleware::Error>> = None;

    for _ in 0u8..3 {
        match (f)().await? {
            Ok(t) => return Ok(t),
            Err(reqwest_middleware::Error::Reqwest(e)) => {
                if e.is_timeout() {
                    response = Some(Err(reqwest_middleware::Error::Reqwest(e)));
                    continue;
                }
                return Err(reqwest_middleware::Error::Reqwest(e));
            }
            Err(otherwise) => {
                return Err(otherwise);
            }
        }
    }

    response.expect("retry http request")
}
