use opentelemetry::{api::Provider, sdk};
use opentelemetry_stackdriver::StackDriverExporter;
use tracing::{span, Level};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{Layer, Registry};

use std::{path::{Path, PathBuf}, thread::sleep, time::Duration};

#[tokio::main]
async fn main() {
  simple_logger::init_with_level(log::Level::Debug).unwrap();
  let args = std::env::args().collect::<Vec<_>>();
  if args.len() < 2 {
    eprintln!("This example requires a path to your stackdriver json credentials as the first argument.");
    return;
  }
  init_tracing(&args[1]).await;
  span!(Level::INFO, "example_span").in_scope(|| {
    sleep(Duration::from_secs(2));
    span!(Level::INFO, "example_child_span").in_scope(|| {
      sleep(Duration::from_secs(2));
    });
  });
  sleep(Duration::from_secs(5));
}

async fn init_tracing(stackdriver_creds: impl AsRef<Path>) {
  StackDriverExporter::connect(stackdriver_creds, PathBuf::from("tokens.json"), &TokioSpawner, None, 5)
    .await
    .map_err(|e| panic!("Error connecting to stackdriver: {:?}", e))
    .and_then(|exporter| {
      tracing::subscriber::set_global_default(
        OpenTelemetryLayer::default()
          .with_tracer(
            sdk::Provider::builder()
              .with_simple_exporter(exporter)
              .build()
              .get_tracer("example"),
          )
          .with_subscriber(Registry::default()),
      )
      .map_err(|e| panic!("Error setting subscriber: {:?}", e))
    })
    .unwrap();
}

use futures::{
  future::FutureObj,
  task::{Spawn, SpawnError},
};

/// For some reason tokio decided not to implement `futures::task::Spawn` anywhere.
/// https://github.com/tokio-rs/tokio/issues/2018
/// So here's a little struct that will do so for you.
pub struct TokioSpawner;

impl Spawn for TokioSpawner {
  fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
    // TODO: check that executor is active; return SpawnError if not.
    tokio::runtime::Handle::current().spawn(future);
    Ok(())
  }
}
