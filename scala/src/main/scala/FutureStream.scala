package cchantep

import scala.concurrent.{ Await, ExecutionContext, Future }

import scala.concurrent.duration.Duration

import akka.NotUsed

import akka.util.ByteString

import akka.stream.IOResult
import akka.stream.scaladsl.{ Sink, Source }

/** See `rust/future_stream.rs` */
object FutureStream {
  type Error = Exception

  type Result[T, E] = Either[E, T]
  type Stream[T] = Source[T, NotUsed]

  def ok[T, E](t: T): Result[T, E] = Right(t)
  def err[E, T](cause: E): Result[T, E] = Left(cause)

  // ---

  def result2try_future[A, B](result: Result[A, Error])(f: A => B): Future[Result[B, Error]] = Future.successful(result.map(f))

  def result2try_stream[A, B](result: Result[A, Error])(f: A => B): Stream[Result[B, Error]] = Source.single(result.map(f))

  def result_map_async[A, B](result: Result[A, Error])(f: A => Future[Result[B, Error]]): Future[Result[B, Error]] =
    result match {
      case Left(error) => Future.successful(Left(error))
      case Right(a) => f(a)
    }

  def result2stream[A, B](result: Result[A, Error])(f: A => B): Stream[B] =
    Source.single(result).flatMapConcat {
      case Left(error) =>
        Source.failed[B](error)

      case Right(a) =>
        Source.single(a).map(f)
    }

  def stream_map_async_with_filter[A, B](stream: Stream[Result[A, Error]])(f: A => Future[Result[B, Error]], isErrored: B => Option[Error])(implicit ec: ExecutionContext): Stream[Result[B, Error]] = {
    stream.mapAsync(1) {
      case Right(a) => f(a).map(_.flatMap { b =>
        isErrored(b) match {
          case Some(error) =>
            Left(error)

          case _ =>
            Right(b)
        }
      })

      case Left(error) =>
        Future.successful(Left(error))
    }
  }

  def stream_map_until[A, B](stream: Stream[A])(f: A => Option[B]): Stream[B] =
    stream.takeWhile(f(_).nonEmpty).collect(Function.unlift(f))

  def stream_map_async_until[A, B](stream: Stream[A])(f: A => Future[Result[B, Error]], isErrored: B => Option[Error]): Stream[B] = stream.mapAsync(1)(f).
    flatMapConcat {
      case Right(b) => isErrored(b) match {
        case Some(error) => Source.failed[B](error) // terminates with error
        case _ => Source.single(b)
      }

      case Left(error) =>
        Source.failed[B](error) // terminates with error
    }

  type Elapsed = scala.concurrent.TimeoutException

  def await_or_timeout[A](future: Future[A], duration: Duration): Result[A, Elapsed] = try {
    Right(Await.result(future, duration))
  } catch {
    case elapsed: Elapsed =>
      Left(elapsed)
  }

  def contramap_sink[A](out: java.io.OutputStream)(f: A => ByteString): Sink[A, Future[IOResult]] = akka.stream.scaladsl.StreamConverters.fromOutputStream(() => out).contramap(f)
}
