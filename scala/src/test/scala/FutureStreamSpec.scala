package cchantep

import scala.concurrent.Future
import scala.concurrent.duration._

import akka.util.ByteString
import akka.stream.scaladsl.{ Sink, Source }

import FutureStream.{ Error, Result }

import org.specs2.concurrent.ExecutionEnv

final class FutureStreamSpec(implicit val ee: ExecutionEnv)
  extends org.specs2.mutable.Specification with SpecWithStream { spec =>

  "future-stream" title

  import spec.{ beLeft => beErr, beRight => beOk }

  // Fixtures
  val okInput: Result[String, Error] = FutureStream.ok("lorem")
  val errInput: Result[String, Error] = FutureStream.err(new Exception("cause"))

  // ---

  "test_result2try_future" should {
    import FutureStream.result2try_future

    "return mapped value on successful input" in {
      result2try_future(okInput)(_.size) must beOk(5).await
    }

    "return error on failed input" in {
      result2try_future(errInput)(_.size) must beErr.await
    }
  }

  "test_result2try_stream" should {
    import FutureStream.result2try_stream

    "return mapped value on successful input" in {
      result2try_stream(okInput)(_.size).runWith(Sink.head) must beOk(5).await
    }

    "return error on failed input" in {
      result2try_stream(errInput)(_.size).runWith(Sink.head) must beErr.await
    }
  }

  "test_result_map_async" should {
    import FutureStream.result_map_async

    def async(value: String): Future[Result[Int, Error]] =
      Future.successful(FutureStream.ok(value.size))

    "return mapped value on successful input" in {
      result_map_async(okInput)(async) must beOk(5).await
    }

    "return error on failed input" in {
      result_map_async(errInput)(async) must beErr.await
    }
  }

  "result2stream" should {
    import FutureStream.result2stream

    "return mapped value on successful input" in {
      result2stream(okInput)(_.size).
        runWith(Sink.head) must beTypedEqualTo(5).await
    }

    "return error on failed input" in {
      result2stream(errInput)(_.size).runWith(Sink.head).
        map(_ => false).recover { case _ => true } must beTrue.await
    }
  }

  "stream_map_until" should {
    import FutureStream.stream_map_until

    "terminate" in {
      val src = Source.fromIterator(
        () => List("lala", "lorem", "rolo").iterator)

      stream_map_until(src) { s =>
        val l = s.size

        if (l % 2 == 0) {
          Some(l)
        } else {
          Option.empty[Int]
        }
      }.runWith(Sink.seq) must beTypedEqualTo(Seq(4 /* "lala" */ )).await
    }
  }

  "stream_map_async_with_filter" should {
    import FutureStream.stream_map_async_with_filter

    def f(s: String): Future[Result[Int, Error]] = {
      val l = s.size

      Future.successful {
        if (l == 0) Left(new Exception("Empty"))
        else Right(l)
      }
    }

    val oddErr = new Exception("Odd")
    def isErrored(sz: Int): Option[Error] = {
      if (sz % 2 == 0) None
      else Some(oddErr)
    }

    // ---

    "skip odd item" in {
      val src = stream_map_async_with_filter(
        Source.single(okInput))(f, isErrored)

      src.runWith(Sink.head[Result[Int, Error]]) must beErr.await
    }

    "succeed even item" in {
      val src = stream_map_async_with_filter(
        Source.single(Right("lala")))(f, isErrored)

      src.runWith(Sink.head[Result[Int, Error]]) must beOk(4).await
    }

    "collect only valid items" in {
      val fooErr = new Exception("Foo")

      val src = stream_map_async_with_filter(
        Source.fromIterator[Result[String, Error]] { () =>
          List(
            Right("lala"),
            Right("lorem"),
            Left(fooErr),
            Right("rololo")).iterator
        })(f, isErrored)

      src.runWith(Sink.seq) must beLike[Seq[Result[Int, Error]]] {
        case collected =>
          collected.headOption must beSome(Right(4 /* "lala" */ )) and {
            collected.drop(1).headOption must beSome(Left(oddErr)) // "lorem"
          } and {
            collected.drop(2).headOption must beSome(Left(fooErr)) // Err
          } and {
            collected.drop(3).headOption must beSome(Right(6 /* "rololo" */ ))
          }
      }.await
    }
  }

  "stream_map_async_until" should {
    import FutureStream.stream_map_async_until

    def f(s: String): Future[Result[Int, Error]] = {
      val l = s.size

      Future.successful {
        if (l == 0) Left(new Exception("Empty"))
        else Right(l)
      }
    }

    val oddErr = new Exception("Odd")
    def isErrored(sz: Int): Option[Error] = {
      if (sz % 2 == 0) None
      else Some(oddErr)
    }

    // ---

    "skip odd item" in {
      def src = stream_map_async_until(Source.single("lorem"))(f, isErrored)

      src.runWith(Sink.headOption[Int]) must throwA(oddErr).await and {
        src.recover {
          case _ => -1
        }.runWith(Sink.headOption[Int]) must beSome(-1).await
      }
    }

    "succeed even item" in {
      val src = stream_map_async_until(Source.single("lala"))(f, isErrored)

      src.runWith(Sink.headOption[Int]) must beSome(4).await
    }

    "collect until valid" in {
      def src = Source.fromIterator(() =>
        List("lala", "lorem", "rololo").iterator)

      stream_map_async_until(src)(f, isErrored).
        runWith(Sink.seq) must throwA(oddErr).await and {
          stream_map_async_until(src)(f, isErrored).recover {
            case _ => -1
          }.runWith(Sink.seq) must beTypedEqualTo(Seq(4, -1)).await
        }
    }
  }

  "await_or_timeout" should {
    import FutureStream.{ Elapsed, await_or_timeout }

    "be successful" in {
      await_or_timeout(Future.successful(1), 3.seconds) must beOk(1)
    }

    "fail with timeout" in {
      val p = scala.concurrent.Promise[Unit]()

      await_or_timeout(p.future, 3.seconds) must beErr.like {
        case _: Elapsed => ok
      }
    }
  }

  "contramap_sink" should {
    import FutureStream.contramap_sink

    "collected converted elements" in {
      val buf = new java.io.ByteArrayOutputStream()
      val res = Source(Seq("foo", "bar", "lorem")).runWith(
        contramap_sink(buf)(ByteString(_)))

      res.map(_ => {}) must beTypedEqualTo({}).await and {
        buf.toString() must_=== "foobarlorem"
      }
    }
  }
}
