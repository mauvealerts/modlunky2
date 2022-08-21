use std::fmt::Debug;

use anyhow::anyhow;
use derivative::Derivative;
use http::{
    header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE},
    HeaderValue, Request, StatusCode, Uri,
};
use hyper::{body::HttpBody, client::HttpConnector, Body};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncWrite, AsyncWriteExt as _},
    sync::watch,
};
use tower::{Service as _, ServiceBuilder, ServiceExt as _};
use tower_http::{
    classify::{
        NeverClassifyEos, ServerErrorsAsFailures, ServerErrorsFailureClass, SharedClassifier,
    },
    follow_redirect::FollowRedirect,
    sensitive_headers::SetSensitiveRequestHeaders,
    trace::{DefaultOnBodyChunk, DefaultOnEos, DefaultOnFailure, ResponseBody},
    ServiceBuilderExt,
};
use tracing::instrument;

type InnerService = SetSensitiveRequestHeaders<
    tower_http::sensitive_headers::SetSensitiveResponseHeaders<
        tower_http::trace::Trace<
            FollowRedirect<hyper::Client<HttpsConnector<HttpConnector>>>,
            SharedClassifier<ServerErrorsAsFailures>,
        >,
    >,
>;

pub type Response = http::Response<
    ResponseBody<
        Body,
        NeverClassifyEos<ServerErrorsFailureClass>,
        DefaultOnBodyChunk,
        DefaultOnEos,
        DefaultOnFailure,
    >,
>;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0:?}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP status: {0:?}")]
    StatusError(StatusCode),
    #[error("HTTP error: {0:?}")]
    GenericHttpError(#[source] anyhow::Error),

    #[error("{0:?}")]
    UnknownError(#[from] anyhow::Error),
}

impl From<http::header::InvalidHeaderValue> for Error {
    fn from(e: http::header::InvalidHeaderValue) -> Error {
        Error::GenericHttpError(e.into())
    }
}

impl From<http::header::ToStrError> for Error {
    fn from(e: http::header::ToStrError) -> Error {
        Error::GenericHttpError(e.into())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::GenericHttpError(e.into())
    }
}

impl From<http::uri::InvalidUri> for Error {
    fn from(e: http::uri::InvalidUri) -> Error {
        Error::GenericHttpError(e.into())
    }
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Error {
        Error::GenericHttpError(e.into())
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Error {
        Error::GenericHttpError(e.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadProgress {
    Waiting(),
    Started(),
    Receiving {
        expected_bytes: Option<u64>,
        received_bytes: u64,
    },
    Finished(),
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct HttpClient {
    #[derivative(Debug = "ignore")]
    service: InnerService,
}

impl HttpClient {
    pub fn new() -> Self {
        {
            let inner_client =
                hyper::client::Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
            let service = ServiceBuilder::new()
                .sensitive_headers([AUTHORIZATION])
                .trace_for_http()
                .follow_redirects()
                .service(inner_client);
            Self { service }
        }
    }

    pub async fn get(&self, uri: &str, auth_token: Option<&String>) -> Result<Response> {
        let uri = uri.parse::<Uri>()?;
        let request = Request::get(uri).version(http::Version::HTTP_11);
        let request = match auth_token {
            Some(token) => {
                let authz_value = "Token ".to_string() + token;
                let authz_header = HeaderValue::from_str(&authz_value)?;
                request.header(AUTHORIZATION, authz_header)
            }
            None => request,
        };

        let request = request
            .body(Body::empty())
            .map_err(|e| Error::UnknownError(e.into()))?;

        let res = self.service.clone().ready().await?.call(request).await?;
        if !res.status().is_success() {
            return Err(Error::StatusError(res.status()));
        }
        Ok(res)
    }

    #[instrument(skip(self, writer, progress))]
    pub async fn download(
        &self,
        uri: &str,
        writer: &mut (impl AsyncWrite + Debug + Send + Unpin),
        progress: &watch::Sender<DownloadProgress>,
    ) -> Result<String> {
        let _ = progress.send(DownloadProgress::Started());
        let mut res = self.get(uri, None).await?;
        let content_type = res
            .headers()
            .get(CONTENT_TYPE)
            .ok_or_else(|| Error::GenericHttpError(anyhow!("No content type for URI {}", uri)))?
            .to_str()?
            .to_string();
        let expected_bytes = if let Some(val) = res.headers().get(CONTENT_LENGTH) {
            Some(val.to_str()?.parse::<u64>()?)
        } else {
            None
        };
        tokio::pin!(writer);
        let mut received_bytes = 0_u64;
        while let Some(chunk) = res.body_mut().data().await {
            let chunk = chunk?;
            received_bytes += chunk.len() as u64;
            let _ = progress.send(DownloadProgress::Receiving {
                expected_bytes,
                received_bytes,
            });
            writer.write_all(&chunk).await?;
        }
        writer.flush().await?;

        let _ = progress.send(DownloadProgress::Finished());
        Ok(content_type)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}
