mod schemas;
mod ya;
// use rig::client::completion::CompletionClientDyn;
use ::rig::completion::Prompt;
use ::rig::completion::message::DocumentSourceKind;
use base64::prelude::*;
// use rig::client::{CompletionClient, ProviderClient};
use rig::client::builder::{ClientFactory, DynClientBuilder};
use rig::message::{Image, ImageMediaType};
use rig::prelude::*;
use std::fs::read;
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
#[tokio::main]
async fn main()
{
  let file_appender = RollingFileAppender::new(
    Rotation::DAILY,
    "./logs",
    "application.log",
  );
  let (non_blocking_appender, _guard) =
    tracing_appender::non_blocking(file_appender);

  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::TRACE)
    .with_target(true)
    .with_writer(non_blocking_appender)
    .init();

  let folder_id = "https://yandex.cloud/ru/docs/resource-manager/operations/folder/get-id";
  let api_key =
    "https://yandex.cloud/ru/docs/iam/concepts/authorization/api-key";
  unsafe {
    std::env::set_var("YANDEX_API_KEY", api_key);
  }

  // build from folder Id (temporary token Auth, require 'yc' configured)
  let ocr1 = ya::Client::from_fldr(folder_id).agent("page").build();

  // build from api key (Api-Key Auth)
  let ocr2 = ya::Client::from_api(api_key).agent("page").build();

  // build from rig dynamic builder (uses Api-Key by default)
  let ocr3 = DynClientBuilder::new()
    .register(ClientFactory::new(
      "yandex",
      ya::Client::from_env_boxed,
      ya::Client::from_val_boxed,
    ))
    .agent("yandex", "page")
    .expect("Could not build")
    .build();

  // usual rig-to-get-image-stuff
  let path = PathBuf::from("docs/spravka.jpg");

  let f = read(path).expect("Could not read file");
  let encoded = BASE64_STANDARD.encode(f);
  let doc = Image {
    data: DocumentSourceKind::Base64(encoded),
    media_type: Some(ImageMediaType::JPEG),
    additional_params: None,
    detail: None,
  };

  // usual rig-agent-prompt stuff
  let resp1 =
    ocr1.prompt(doc.clone()).await.expect("Could not infer");
  println!("{}", resp1);

  let resp2 =
    ocr2.prompt(doc.clone()).await.expect("Could not infer");
  println!("{}", resp2);

  let resp3 = ocr3.prompt(doc).await.expect("Could not infer");
  println!("{}", resp3);
}
