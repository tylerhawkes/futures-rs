use super::assert_future;
use crate::FutureExt;
use core::pin::Pin;
use futures_core::future::{FusedFuture, Future};
use futures_core::task::{Context, Poll};

/// Future for the [`immediate`](immediate()) function.
#[derive(Debug, Clone)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Immediate<T>(T);

impl<T, F> Future for Immediate<F>
where
    F: Future<Output = T>,
{
    type Output = Option<T>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.0) };
        match inner.poll(cx) {
            Poll::Ready(t) => Poll::Ready(Some(t)),
            Poll::Pending => Poll::Ready(None),
        }
    }
}

/// Creates a future that is immediately ready with an Option of a value.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures::future;
///
/// let r = future::immediate(future::ready(1));
/// assert_eq!(r.await, Some(1));
/// let p = future::immediate(future::pending::<i32>());
/// assert_eq!(p.await, None);
/// # });
/// ```
pub fn immediate<F: Future>(f: F) -> Immediate<F> {
    assert_future::<Option<F::Output>, Immediate<F>>(Immediate(f))
}

/// Future for the [`immediate_unpin`](immediate_unpin()) function.
#[derive(Debug, Clone)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ImmediateUnpin<T>(Option<T>);

impl<T, F> Future for ImmediateUnpin<F>
where
    F: Future<Output = T> + Unpin,
{
    type Output = Result<T, F>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<T, F>> {
        // SAFETY: `get_unchecked_mut` is never used to move the `Option` inside `self`.
        // `x` is guaranteed to be pinned because it comes from `self` which is pinned.
        let mut inner = self.get_mut().0.take().expect("ImmediateUnpin polled after completion");
        match inner.poll_unpin(cx) {
            Poll::Ready(t) => Poll::Ready(Ok(t)),
            Poll::Pending => Poll::Ready(Err(inner)),
        }
    }
}

impl<T: Future + Unpin> FusedFuture for ImmediateUnpin<T> {
    fn is_terminated(&self) -> bool {
        self.0.is_none()
    }
}

/// Creates a future that is immediately ready with an Option of a value.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures::future;
///
/// let r = future::immediate_unpin(future::ready(1_i32));
/// assert_eq!(r.await.unwrap(), 1);
/// // futures::pending!() returns pending once and then evaluates to `()`
/// let p = future::immediate_unpin(Box::pin(async { futures::pending!() }));
/// match p.await {
///     Ok(_) => unreachable!(),
///     Err(e) => {
///         e.await;
///     }
/// }
/// # });
/// ```
pub fn immediate_unpin<F: Future + Unpin>(f: F) -> ImmediateUnpin<F> {
    assert_future::<Result<F::Output, _>, ImmediateUnpin<F>>(ImmediateUnpin(Some(f)))
}
