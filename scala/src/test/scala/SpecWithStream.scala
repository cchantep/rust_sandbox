package cchantep

import org.specs2.mutable.Specification

import org.specs2.concurrent.ExecutionEnv

trait SpecWithStream { spec: Specification { def ee: ExecutionEnv } =>
  protected implicit val system = akka.actor.ActorSystem(
    name = spec.getClass.getSimpleName,
    defaultExecutionContext = Some(ee.ec)) // TODO: shutdown

  implicit lazy val materializer =
    akka.stream.Materializer.createMaterializer(system)
}
