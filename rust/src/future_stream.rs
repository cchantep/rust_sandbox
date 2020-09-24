/// Future and Stream experimentations.
///
/// **Usage:**
///
/// Implements the following functions and make the provided tests :satisfied:.
///
/// > These rustlings can be checkout locally from [GitHub](https://github.com/cchantep/rust_sandbox).
///
/// *See [solutions](https://github.com/cchantep/rust_sandbox/blob/solution/rust/src/future_stream.rs)*
///
/// **Requirements:** (`Cargo.toml`)
///
/// ```toml
/// [dependencies]
/// futures = "0.3"
/// tokio = { version = "0.2", features = [ "time" ] }
/// ```
extern crate futures; // 0.3.5

use std::convert::AsRef;
use std::io::Error;
use std::result::Result;
use std::time::Duration;

use futures::future::{ready, Future};

use futures::io::AsyncWrite;
use futures::sink::Sink;
use futures::stream::Stream;

/// Function `result2try_future` with at least 2 type parameters `A` and `B`;
/// Returns a `Future` containing the input `result` mapped using `f`.
///
/// # Arguments
///
/// * `result` - The input `Result<A, Error>`
/// * `f` - The function applied within the `result` if and only if it's `Ok`: `fn(A) -> B`
///
#[allow(dead_code)]
fn result2try_future<A, B>(
    result: Result<A, Error>,
    f: fn(A) -> B,
) -> impl Future<Output = Result<B, Error>> {
    ready(result.map(f))
}

/// Function `result2try_stream` with at least 2 type parameters `A` and `B`;
/// Returns a `Future` containing the input `result` mapped using `f`.
///
/// # Arguments
///
/// * `result` - The input `Result<A, Error>`
/// * `f` - The function applied within the `result` if and only if it's `Ok`
///
#[allow(dead_code)]
fn result2try_stream<'a, A: 'a, B: 'a>(
    result: Result<A, Error>,
    f: fn(A) -> B,
) -> impl Stream<Item = Result<B, Error>> + 'a {
    futures::stream::once(async move { result.map(f) })
}

/// Function `result_map_async` with at least 2 type parameters `A` and `B`;
/// Returns a `Future` containing the input `result` mapped using `f`.
///
/// # Arguments
///
/// * `result` - The input `Result<A, Error>`
/// * `f` - The asynchronous function applied within the `result` if and only if it's `Ok`: `fn(A) -> Future<Output = Result<B, Error>>`
///
#[allow(dead_code)]
fn result_map_async<A, B, Fut: Future<Output = Result<B, Error>>>(
    result: Result<A, Error>,
    f: fn(A) -> Fut,
) -> impl Future<Output = Result<B, Error>> {
    use futures::future::FutureExt; // provide '{left,right}_future'

    match result {
        Ok(a) => f(a).right_future(),
        Err(cause) => ready(Err(cause)).left_future(),
    }
}

/// Function `result2stream` with at least 2 type parameters `A` and `B`;
/// Returns a `Stream` containing the input `result` mapped using `f`.
///
/// !! The `Stream` should stops as soon as an `Err`
/// item is encountered in `result`.
///
/// # Arguments
///
/// * `result` - The input `Result<A, Error>`
/// * `f` - The function applied within `result` if and only if it's `Ok`: `fn(A) -> B`
///
#[allow(dead_code)]
fn result2stream<'a, A: 'a, B: 'a>(
    result: Result<A, Error>,
    f: fn(A) -> B,
) -> impl Stream<Item = B> + 'a {
    use futures::stream::StreamExt; // provide 'filter_map'

    futures::stream::once(async move { result.map(f) }).filter_map(|item| {
        match item {
            Ok(v) => ready(Some(v)),
            Err(_cause) => ready(None), // !!
        }
    })
}

/// Function `stream_map_async_with_filter`
/// with at least 2 type parameters `A` and `B`;
///
/// Returns a `Stream` containing the input `stream` mapped
/// from `A` to `Result<B, Error>` using `f`.
///
/// Each `B` value is also checked with the `is_errored` function.
///
/// * If `Some(error)` is returned, then the `B` value should be mapped to `Err(error)`;
/// * Otherwise `Ok(b_value)` is kept.
///
/// # Arguments
///
/// * `stream` - The input `Stream<Result<A, Error>>`
/// * `f` - The function applied to the `stream` items if and only if it's `Ok`: `fn(A) -> B`
///
#[allow(dead_code)]
fn stream_map_async_with_filter<
    'a,
    A: 'a,
    I: 'a + Stream<Item = Result<A, Error>>,
    B: 'a + Copy,
    Fut: 'a + Future<Output = Result<B, Error>>,
>(
    stream: I,
    f: fn(A) -> Fut,
    is_errored: fn(B) -> Option<Error>,
) -> impl Stream<Item = Result<B, Error>> + 'a {
    use futures::stream::TryStreamExt; // provide 'and_then'

    stream
        .and_then(f)
        .and_then(move |b| ready(is_errored(b).map_or_else(|| Ok(b), |cause| Err(cause))))
}

/// Function `stream_map_until` with at least 2 type parameters `A` and `B`.
///
/// Each `A` item is mapped as `B` using the `f` function.
/// If `Some` mapped value is returned, it's emitted as a `Stream` item;
/// Other if `None` is returned, the `Stream` terminates.
///
/// # Arguments
///
/// * `stream` - The input `Stream<A>`
/// * `f` - The function applied to the `stream` items: `fn(A) -> Option<B>`
///
#[allow(dead_code)]
fn stream_map_until<'a, A: 'a + Copy, B: 'a, I: 'a + Stream<Item = A>>(
    stream: I,
    f: fn(A) -> Option<B>,
) -> impl Stream<Item = B> + 'a {
    use futures::stream::StreamExt; // provide 'take_while', 'filter_map'

    stream
        .take_while(move |a| ready(f(*a).is_some()))
        .filter_map(move |a| ready(f(a)))
}

/// Function `stream_map_async_until`
/// with at least 2 type parameters `A` and `B`.
///
/// Returns a `Stream` containing the input `stream` mapped
/// from `A` to `Result<B, Error>` using `f`;
/// If `Err(someError)` is returned, then the stream terminates.
///
/// Each `B` value is also checked with `is_errored` function.
///
/// * If `Some(error)` is returned, then the stream terminates;
/// * Otherwise the `B` value that's `Ok` is pushed as a `Stream` item.
///
/// # Arguments
///
/// * `stream` - The input `Stream<A>`
/// * `f` - The function applied to the `stream` items if and only if it's `Ok`: `fn(A) -> B`
///
#[allow(dead_code)]
fn stream_map_async_until<
    'a,
    A: 'a,
    I: 'a + Stream<Item = A>,
    B: 'a + Copy,
    Fut: 'a + Future<Output = Result<B, Error>>,
>(
    stream: I,
    f: fn(A) -> Fut,
    is_errored: fn(B) -> Option<Error>,
) -> impl Stream<Item = B> + 'a {
    // provide 'then', 'take_while', 'filter_map'
    use futures::stream::StreamExt;

    stream
        .then(f)
        .take_while(|try_b| ready(try_b.is_ok()))
        .filter_map(|try_b| match try_b {
            Ok(b) => ready(Some(b)),
            Err(_cause) => ready(None), // !!
        })
        .take_while(move |b| ready(is_errored(*b).map_or_else(|| true, |_cause| false /* !! */)))
}

/// Function `await_or_timeout` with at least 1 type parameter `A`;
///
/// Returns the awaited `Ok(a_value)`, or timeout `Err`.
///
/// # Arguments
///
/// * `future` - The asynchronous `A` value
/// * `duration` - The timeout duration
///
#[allow(dead_code)]
async fn await_or_timeout<A, Fut: Future<Output = A>>(
    future: Fut,
    duration: Duration,
) -> Result<A, tokio::time::Elapsed> {
    tokio::time::timeout(duration, future).await
}

/// Function `contramap_sink` with at least 2 type parameters `A` and `B`;
///
/// Returns a `Sink` that consumes `A` items, then converts each item as `B`
/// before writing to the `output`.
///
/// # Arguments
///
/// * `f` - The function that converts a `A` value into a `B` one
/// * `output` - The sink output
///
#[allow(dead_code)]
fn contramap_sink<'a, A: 'a, B: 'a + AsRef<[u8]>, Out: 'a + AsyncWrite>(
    f: fn(A) -> B,
    output: Out,
) -> impl Sink<A, Error = Error> + 'a {
    use futures::io::AsyncWriteExt;
    use futures::sink::SinkExt; // provide 'with'

    output.into_sink::<B>().with(move |a| ready(Ok(f(a))))
}

// ---

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::pending;
    use std::io::ErrorKind;

    // Test runtime
    use tokio::runtime::Runtime;

    // Fixtures
    fn ok_input() -> Result<String, Error> {
        Ok(String::from("lorem"))
    }

    fn err_input() -> Result<String, Error> {
        Err(Error::new(ErrorKind::Other, "cause"))
    }

    #[test]
    fn test_result2try_future() {
        let mut rt: Runtime = Runtime::new().unwrap();

        assert_eq!(
            rt.block_on(result2try_future(ok_input(), |s| s.len()))
                .unwrap(),
            5
        );

        assert_eq!(
            rt.block_on(result2try_future(err_input(), |s| s.len()))
                .is_err(),
            true
        );
    }

    #[test]
    fn test_result2try_stream() {
        let mut rt: Runtime = Runtime::new().unwrap();

        use futures::stream::StreamExt; // provide 'next'

        let ok_head: Option<Result<usize, Error>> = rt
            .block_on(
                result2try_stream(ok_input(), |s| s.len()).collect::<Vec<Result<usize, Error>>>(),
            )
            .pop();

        assert_eq!(ok_head.unwrap().unwrap(), 5);

        let err_head: Option<Result<usize, Error>> = rt
            .block_on(
                result2try_stream(err_input(), |s| s.len()).collect::<Vec<Result<usize, Error>>>(),
            )
            .pop();

        assert_eq!(err_head.unwrap().is_err(), true);
    }

    #[test]
    fn test_result_map_async() {
        let mut rt: Runtime = Runtime::new().unwrap();

        assert_eq!(
            rt.block_on(result_map_async(ok_input(), |s| ready(Ok(s.len()))))
                .unwrap(),
            5
        );

        assert_eq!(
            rt.block_on(result_map_async(err_input(), |s| ready(Ok(s.len()))))
                .is_err(),
            true
        );
    }

    #[test]
    fn test_result2stream() {
        let mut rt: Runtime = Runtime::new().unwrap();

        use futures::stream::StreamExt;

        let ok_head: Option<usize> = rt
            .block_on(result2stream(ok_input(), |s| s.len()).collect::<Vec<usize>>())
            .pop();

        assert_eq!(ok_head.unwrap(), 5);

        let err_head: Option<usize> = rt
            .block_on(result2stream(err_input(), |s| s.len()).collect::<Vec<usize>>())
            .pop();

        assert_eq!(err_head.is_none(), true);
    }

    #[test]
    fn test_stream_map_until() {
        let mut rt: Runtime = Runtime::new().unwrap();

        use futures::stream::{iter, StreamExt};

        let mut usize_stream = stream_map_until(iter(vec!["lala", "lorem", "rolo"]), |s| {
            let l = s.len();

            if l % 2 == 0 {
                Some(l)
            } else {
                None
            }
        });

        assert_eq!(rt.block_on(usize_stream.next()).unwrap(), 4); // "lala"

        assert_eq!(rt.block_on(usize_stream.next()).is_none(), true); // "lorem".len() % 2 != 0

        assert_eq!(rt.block_on(usize_stream.next()).is_none(), true); // "rolo"; stream terminated
    }

    #[test]
    fn test_stream_map_async_with_filter() {
        let mut rt: Runtime = Runtime::new().unwrap();

        fn f(s: String) -> impl Future<Output = Result<usize, Error>> {
            let l = s.len();

            ready(if l == 0 {
                Err(Error::new(ErrorKind::Other, "Empty"))
            } else {
                Ok(l)
            })
        }

        fn is_errored(sz: usize) -> Option<Error> {
            if sz % 2 == 0 {
                None
            } else {
                Some(Error::new(ErrorKind::Other, "Odd"))
            }
        }

        // ---

        use futures::stream::{iter, once, StreamExt};

        let ok_head = rt
            .block_on(
                stream_map_async_with_filter(once(ready(ok_input())), f, is_errored)
                    .collect::<Vec<Result<usize, Error>>>(),
            )
            .pop()
            .unwrap();

        assert_eq!(ok_head.is_err(), true); // "lorem".len() % 2 != 0

        let head2 = rt
            .block_on(
                stream_map_async_with_filter(once(ready(Ok(String::from("lala")))), f, is_errored)
                    .collect::<Vec<Result<usize, Error>>>(),
            )
            .pop()
            .unwrap()
            .unwrap();

        assert_eq!(head2, 4);

        let mut vec_stream1 = stream_map_async_with_filter(
            iter(vec![
                Ok(String::from("lala")),
                Ok(String::from("lorem")),
                Err(Error::new(ErrorKind::Other, "Foo")),
                Ok(String::from("rololo")),
            ]),
            f,
            is_errored,
        );

        assert_eq!(rt.block_on(vec_stream1.next()).unwrap().unwrap(), 4); // "lala"

        assert_eq!(rt.block_on(vec_stream1.next()).unwrap().is_err(), true); // "lorem"

        assert_eq!(rt.block_on(vec_stream1.next()).unwrap().is_err(), true); // Err

        assert_eq!(rt.block_on(vec_stream1.next()).unwrap().unwrap(), 6); // "rololo"
    }

    #[test]
    fn test_stream_map_async_until() {
        let mut rt: Runtime = Runtime::new().unwrap();

        fn f(s: String) -> impl Future<Output = Result<usize, Error>> {
            let l = s.len();

            ready(if l == 0 {
                Err(Error::new(ErrorKind::Other, "Empty"))
            } else {
                Ok(l)
            })
        }

        fn is_errored(sz: usize) -> Option<Error> {
            if sz % 2 == 0 {
                None
            } else {
                Some(Error::new(ErrorKind::Other, "Odd"))
            }
        }

        // ---

        use futures::stream::{iter, once, StreamExt};

        let ok_head = rt
            .block_on(
                stream_map_async_until(once(ready(String::from("lorem"))), f, is_errored)
                    .collect::<Vec<usize>>(),
            )
            .pop();

        assert_eq!(ok_head.is_none(), true); // "lorem".len() % 2 != 0

        let head2 = rt
            .block_on(
                stream_map_async_until(once(ready(String::from("lala"))), f, is_errored)
                    .collect::<Vec<usize>>(),
            )
            .pop()
            .unwrap();

        assert_eq!(head2, 4);

        let mut vec_stream1 = stream_map_async_until(
            iter(vec![
                String::from("lala"),
                String::from("lorem"),
                String::from("rololo"),
            ]),
            f,
            is_errored,
        );

        assert_eq!(rt.block_on(vec_stream1.next()).unwrap(), 4); // "lala"

        assert_eq!(rt.block_on(vec_stream1.next()).is_none(), true); // "lorem"

        assert_eq!(rt.block_on(vec_stream1.next()).is_none(), true); // stream terminated

        assert_eq!(rt.block_on(vec_stream1.next()).is_none(), true); // "rololo" - stream terminated
    }

    #[test]
    fn test_await_or_timeout() {
        let mut rt: Runtime = Runtime::new().unwrap();
        let tmout = Duration::new(5, 0);

        assert_eq!(rt.block_on(await_or_timeout(ready(1), tmout)).unwrap(), 1);

        assert_eq!(
            rt.block_on(await_or_timeout(pending::<u8>(), tmout))
                .is_err(),
            true
        );
    }

    #[test]
    fn test_contramap_sink() {
        use futures::stream::{self, StreamExt}; // provide 'forward'

        let mut rt: Runtime = Runtime::new().unwrap();

        let mut out = vec![];

        fn f<'a>(s: &'a str) -> &'a [u8] {
            s.as_bytes()
        }

        let source = stream::iter(vec![Ok("foo"), Ok("bar"), Ok("lorem")]);

        let conv_sink = contramap_sink(f, &mut out);

        let mat_val = rt.block_on(source.forward(conv_sink));

        assert_eq!(mat_val.is_ok(), true);

        assert_eq!(
            out,
            vec![102, 111, 111, 98, 97, 114, 108, 111, 114, 101, 109]
        );
    }
}
