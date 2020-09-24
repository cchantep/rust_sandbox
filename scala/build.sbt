organization := "org.github.cchantep"

name := "rust-equivalent"

version := "1.0.0"

libraryDependencies ++= Seq(
  "com.typesafe.akka" %% "akka-stream" % "2.6.9")

libraryDependencies ++= Seq("core", "junit").map { n =>
  "org.specs2" %% s"specs2-${n}" % "4.10.3" % Test
}

