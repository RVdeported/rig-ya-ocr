// yandex-ocr API client and Rig integration
use reqwest::Client as HttpClient;
use rig::client::{
  CompletionClient, ProviderClient, VerifyClient, VerifyError,
};
use rig::completion::{
  self, CompletionError, CompletionRequest, GetTokenUsage,
};
use rig::message::{AssistantContent, DocumentSourceKind, MimeType};
use rig::{OneOrMany, impl_conversion_traits, message};
use serde::{Deserialize, Serialize};
use std::process::Command;

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use crate::schemas::*;
use chrono::{Local, NaiveDateTime, TimeDelta};
use regex::Regex;

// ================================================================
// Main Yandex Client
// ================================================================

const YA_OCR_TOKEN_UPD: TimeDelta = TimeDelta::try_hours(3).unwrap();
const YA_BASE_URL: &'static str =
  "https://ocr.api.cloud.yandex.net/ocr/v1";
const YA_TOKEN_PATTERN: &'static str =
  "t1\\.[A-Z0-9a-z_-]+[=]{0,2}\\.[A-Z0-9a-z_-]{86}[=]{0,2}";

// -------------------------------------------------//
// Miscalennious                                    //
// -------------------------------------------------//
#[derive(PartialEq, Clone, Debug)]
pub enum AuthType
{
  Token,
  ApiKey,
  None,
}

#[derive(Debug)]
pub enum YaErr
{
  TokenUpdErr(String),
  BuildErr(String),
  ReqErr(String),
}

impl Display for YaErr
{
  fn fmt(&self, f: &mut Formatter) -> FmtResult
  {
    match self {
      YaErr::TokenUpdErr(e) => {
        write!(f, "Error on Token update: {}", e)
      }
      YaErr::BuildErr(e) => {
        write!(f, "Error on construct: {}", e)
      }
      YaErr::ReqErr(e) => {
        write!(f, "Error on request: {}", e)
      }
    }
  }
}

impl Error for YaErr {}

// -------------------------------------------------//
// ClientBuilder                                    //
// -------------------------------------------------//
#[derive(Clone)]
pub struct Client
{
  base_url: String,
  api_key: Option<String>,
  token: Option<String>,
  folder: Option<String>,
  token_upd: Option<NaiveDateTime>,
  rx: Regex,
  auth_t: AuthType,
  http_client: HttpClient,
  pub langs: Vec<String>,
}

impl Client
{
  pub fn from_full(
    a_base_url: Option<String>,
    a_api_key: Option<String>,
    a_token: Option<String>,
    a_folder: Option<String>,
    a_tkn_pattern: Option<&str>,
    a_http_cli: Option<HttpClient>,
    a_langs: Option<Vec<String>>,
  ) -> Result<Self, YaErr>
  {
    // deduction of authh type
    let auth_t = if a_api_key.is_some() {
      AuthType::ApiKey
    } else if a_folder.is_some() {
      AuthType::Token
    } else {
      AuthType::None
    };

    match auth_t {
      AuthType::None => {
        return Err(YaErr::BuildErr(
          "Incorrect auth details: need Api-Key or folder id"
            .to_string(),
        ));
      }
      _ => {}
    }

    let http_client = if let Some(http_client) = a_http_cli {
      http_client
    } else {
      HttpClient::builder()
        .build()
        .expect("Not valid http client")
    };

    let mut out = Self {
      base_url: a_base_url.unwrap_or(YA_BASE_URL.to_string()),
      api_key: a_api_key,
      token: a_token.clone(),
      folder: a_folder,
      token_upd: if a_token.is_some() {
        Some(Local::now().naive_local())
      } else {
        None
      },
      rx: Regex::new(a_tkn_pattern.unwrap_or(YA_TOKEN_PATTERN))
        .unwrap(),
      auth_t: auth_t.clone(),
      http_client: http_client,
      langs: a_langs.unwrap_or(vec!["ru".to_string()]),
    };

    if out.auth_t == AuthType::Token {
      out.upd_token()?;
    }

    tracing::trace!("Created Ocr with params: {:?}", out);

    return Ok(out);
  }

  pub fn from_fldr(a_fldr: &str) -> Self
  {
    return Self::from_full(
      None,
      None,
      None,
      Some(a_fldr.to_string()),
      None,
      None,
      None,
    )
    .expect("Could not build Yandex client");
  }

  pub fn from_api(a_api: &str) -> Self
  {
    return Self::from_full(
      None,
      Some(a_api.to_string()),
      None,
      None,
      None,
      None,
      None,
    )
    .expect("Could not build Yandex client");
  }

  pub fn new(api_key: &str) -> Self
  {
    Self::from_full(
      None,
      Some(api_key.to_string()),
      None,
      None,
      None,
      None,
      None,
    )
    .expect("Could not create Yandex OCR")
  }

  pub fn base_url(mut self, base_url: &str) -> Self
  {
    self.base_url = base_url.to_string();
    self
  }

  pub fn custom_client(mut self, client: reqwest::Client) -> Self
  {
    self.http_client = client;
    self
  }

  //================================================//
  // Token upd                                      //
  //================================================//
  fn upd_token(&mut self) -> Result<(), YaErr>
  {
    let now: NaiveDateTime = Local::now().naive_local();
    if self.token_upd.is_some() && self.token.is_some() {
      let delta: TimeDelta = now - self.token_upd.unwrap();
      if delta < YA_OCR_TOKEN_UPD {
        tracing::debug!(
          "YaOcr::upd_token: not required to upd, last token updated {:?}",
          self.token_upd.unwrap()
        );
        return Ok(());
      }
    }

    let output = Command::new("bash")
      .arg("-c")
      .arg("yc iam create-token")
      .output();

    if output.is_err() {
      return Err(YaErr::TokenUpdErr(
        "Error on bash script".to_string(),
      ));
    }

    let mut tkn = match String::from_utf8(output.unwrap().stdout) {
      Ok(t) => t,
      Err(e) => {
        return Err(YaErr::TokenUpdErr(format!(
          "Error on stdout read {}",
          e
        )));
      }
    };

    tkn.pop();

    if !self.rx.is_match(tkn.as_str()) {
      return Err(YaErr::TokenUpdErr(format!(
        "Not valid token: {}",
        tkn
      )));
    }

    tracing::debug!("Token has been upgraded {}", tkn.clone());
    self.token = Some(tkn);
    self.token_upd = Some(now);

    return Ok(());
  }
}

impl std::fmt::Debug for Client
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    f.debug_struct("Client")
      .field("base_url", &self.base_url)
      .field("http_client", &self.http_client)
      .field("api_key", &"<REDACTED>")
      .finish()
  }
}

impl Client
{
  //-----------------------------------------------//
  // get, post utilities                           //
  //-----------------------------------------------//
  pub fn post(&mut self, path: &str) -> reqwest::RequestBuilder
  {
    let url =
      format!("{}/{}", self.base_url, path).replace("//", "/");

    match self.auth_t {
      AuthType::Token => {
        self.upd_token().expect("Could not renew token");

        self
          .http_client
          .post(url)
          .header("x-folder-id", self.folder.clone().unwrap())
          .header("x-data-logging-enabled", "true")
          .bearer_auth(self.token.clone().unwrap())
      }
      AuthType::ApiKey => self
        .http_client
        .post(url)
        .header("x-data-logging-enabled", "true")
        .header(
          "Authorization",
          format!("Api-Key {}", self.api_key.clone().unwrap()),
        ),
      AuthType::None => {
        panic!("Auth type for yaOcr is not defined");
      }
    }
  }

  // pub fn get(&mut self, path: &str) -> reqwest::RequestBuilder
  // {
  //   let url =
  //     format!("{}/{}", self.base_url, path).replace("//", "/");
  //
  //   match self.auth_t {
  //     AuthType::Token => {
  //       self.upd_token().expect("Could not renew token");
  //
  //       self
  //         .http_client
  //         .get(url)
  //         .header("x-data-logging-enabled", "true")
  //         .header("x-folder-id", self.folder.clone().unwrap())
  //         .bearer_auth(self.token.clone().unwrap())
  //     }
  //     AuthType::ApiKey => self
  //       .http_client
  //       .get(url)
  //       .header("x-data-logging-enabled", "true")
  //       .header(
  //         "Authorization",
  //         format!("Api-Key {}", self.api_key.clone().unwrap()),
  //       ),
  //     AuthType::None => {
  //       panic!("Auth type for yaOcr is not defined");
  //     }
  //   }
  // }
}

impl ProviderClient for Client
{
  // If you prefer the environment variable approach:
  fn from_env() -> Self
  {
    let api_key = std::env::var("YANDEX_API_KEY")
      .expect("YANDEX_API_KEY not set");
    Self::new(&api_key)
  }

  fn from_val(input: rig::client::ProviderValue) -> Self
  {
    let rig::client::ProviderValue::Simple(api_key) = input else {
      panic!("Incorrect provider value type")
    };
    Self::new(&api_key)
  }
}

impl CompletionClient for Client
{
  type CompletionModel = CompletionModel;

  /// Creates a Yandex OCR model
  fn completion_model(&self, model_name: &str) -> CompletionModel
  {
    CompletionModel {
      client: self.clone(),
      model: model_name.to_string(),
    }
  }
}

impl VerifyClient for Client
{
  // #[cfg_attr(feature = "worker", worker::send)]
  async fn verify(&self) -> Result<(), VerifyError>
  {
    return Ok(());
  }
}

impl_conversion_traits!(
    AsEmbeddings,
    AsTranscription,
    AsImageGeneration,
    AsAudioGeneration for Client
);

#[derive(Debug, Deserialize)]
struct ApiErrorResponse
{
  message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ApiResponse<T>
{
  Ok(T),
  Err(ApiErrorResponse),
}

impl From<ApiErrorResponse> for CompletionError
{
  fn from(err: ApiErrorResponse) -> Self
  {
    CompletionError::ProviderError(err.message)
  }
}

/// The response shape from the Yandex API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletionResponse
{
  pub result: ResultOcr,
}

/// The struct implementing the `CompletionModel` trait
#[derive(Clone)]
pub struct CompletionModel
{
  pub client: Client,
  pub model: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct YaCompletionRequest
{
  #[serde(rename = "mimeType")]
  mime_type: String,

  #[serde(rename = "languageCodes")]
  language_codes: Vec<String>,
  model: String,
  content: String, // base64 encoded
}

impl completion::CompletionModel for CompletionModel
{
  type Response = CompletionResponse;
  type StreamingResponse = CompletionResponse;

  // #[cfg_attr(feature = "worker", worker::send)]
  async fn completion(
    &self,
    completion_request: CompletionRequest,
  ) -> Result<
    completion::CompletionResponse<CompletionResponse>,
    rig::completion::CompletionError,
  >
  {
    // if completion_request.documents.len() != 1 {
    //   return Err(CompletionError::RequestError(Box::new(
    //     YaErr::ReqErr(format!(
    //       "Can only send one document at a time {:?}",
    //       completion_request
    //     )),
    //   )));
    // }

    let docs_msg = match completion_request.chat_history.first() {
      message::Message::User { content } => {
        // filter only documents
        let docs = content
          .into_iter()
          .filter_map(|c| match c {
            message::UserContent::Document(doc) => Some(doc),
            _ => None,
          })
          .collect::<Vec<_>>();

        docs
      }

      _ => {
        return Err(CompletionError::RequestError(Box::new(
          YaErr::ReqErr("Can only send documents".to_string()),
        )));
      }
    };

    let imgs_msg = match completion_request.chat_history.first() {
      message::Message::User { content } => {
        // filter only documents
        let docs = content
          .into_iter()
          .filter_map(|c| match c {
            message::UserContent::Image(doc) => Some(doc),
            _ => None,
          })
          .collect::<Vec<_>>();

        docs
      }

      _ => {
        return Err(CompletionError::RequestError(Box::new(
          YaErr::ReqErr("Can only send documents".to_string()),
        )));
      }
    };

    let c;
    let mime_t;
    if docs_msg.len() > 0 {
      c = Some(match docs_msg[0].data.clone() {
        DocumentSourceKind::Base64(s) => s,
        _ => {
          return Err(CompletionError::RequestError(Box::new(
            YaErr::ReqErr("Should be base64 encoded".to_string()),
          )));
        }
      });
      mime_t = Some(
        docs_msg[0]
          .clone()
          .media_type
          .unwrap()
          .to_mime_type()
          .to_string(),
      );
    } else if imgs_msg.len() > 0 {
      c = Some(match imgs_msg[0].data.clone() {
        DocumentSourceKind::Base64(s) => s,
        _ => {
          return Err(CompletionError::RequestError(Box::new(
            YaErr::ReqErr("Should be base64 encoded".to_string()),
          )));
        }
      });
      mime_t = Some(
        imgs_msg[0]
          .clone()
          .media_type
          .unwrap()
          .to_mime_type()
          .to_string(),
      );
    } else {
      c = None;
      mime_t = None;
    }

    let c_f;
    let mime_t_f;
    match c {
      Some(s) => {
        c_f = s;
        mime_t_f = mime_t.unwrap();
      }
      None => {
        return Err(CompletionError::RequestError(Box::new(
          YaErr::ReqErr(
            "Incorrect msg - required Image or Doc".to_string(),
          ),
        )));
      }
    };

    let request = YaCompletionRequest {
      mime_type: mime_t_f,
      language_codes: self.client.langs.clone(),
      model: self.model.clone(),
      content: c_f,
    };

    tracing::trace!("Yandex completion request: {:?}", &request);

    let response;
    unsafe {
      let cli = &self.client as *const Client as *mut Client;
      let bld =
        <*mut Client>::as_mut(cli).unwrap().post("/recognizeText");

      response = bld.json(&request).send().await?;
    }

    if response.status().is_success() {
      let t = response.text().await?;
      tracing::trace!(target: "rig", "Yandex completion: {}", t);

      match serde_json::from_str::<ApiResponse<CompletionResponse>>(
        &t,
      )? {
        ApiResponse::Ok(response) => {
          tracing::trace!("ready to try_into");
          response.try_into()
        }
        ApiResponse::Err(err) => {
          Err(CompletionError::ProviderError(err.message))
        }
      }
    } else {
      Err(CompletionError::ProviderError(response.text().await?))
    }
  }

  async fn stream(
    &self,
    _: CompletionRequest,
  ) -> Result<
    rig::streaming::StreamingCompletionResponse<
      Self::StreamingResponse,
    >,
    CompletionError,
  >
  {
    return Err(CompletionError::RequestError(Box::new(
      YaErr::ReqErr("Cannot send streaming request".to_string()),
    )));
  }
}

impl GetTokenUsage for CompletionResponse
{
  fn token_usage(&self) -> Option<rig::completion::Usage>
  {
    let mut usage = rig::completion::Usage::new();
    usage.input_tokens = 0;
    usage.output_tokens = 0;
    usage.total_tokens = 0;

    Some(usage)
  }
}

impl TryFrom<CompletionResponse>
  for completion::CompletionResponse<CompletionResponse>
{
  type Error = CompletionError;

  fn try_from(
    response: CompletionResponse,
  ) -> Result<Self, Self::Error>
  {
    tracing::trace!("TRYING FROM");
    let choice = OneOrMany::one(AssistantContent::text(format!(
      "{}\n\n{:?}",
      response.result.text_ann.full_text.clone(),
      response.result.text_ann.entities
    )));
    let usage = completion::Usage {
      input_tokens: 0,
      output_tokens: 0,
      total_tokens: 0,
    };

    Ok(completion::CompletionResponse {
      choice,
      usage,
      raw_response: response,
    })
  }
}

// ================================================================
// DeepSeek Completion API
// ================================================================

// /// `deepseek-chat` completion model
// pub const DEEPSEEK_CHAT: &str = "deepseek-chat";
// /// `deepseek-reasoner` completion model
// pub const DEEPSEEK_REASONER: &str = "deepseek-reasoner";
