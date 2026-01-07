use futures::future::{select, Either};
use futures_timer::Delay;
use std::time::Duration;

pub async fn with_timeout<F, T>(fut: F, dur: Duration) -> Result<T, ()>
where
    F: std::future::Future<Output = T> + Unpin,
{
    match select(fut, Delay::new(dur)).await {
        Either::Left((val, _delay_future)) => Ok(val),
        Either::Right((_unit, _original_future)) => Err(()),
    }
}
