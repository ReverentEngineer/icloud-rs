use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Method, Request, Response, StatusCode};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use serde_json::json;
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};

mod uuid;
use crate::error::Error;
use crate::drive::DriveService;

const GLOBAL_HEADERS: [(&'static str, &'static str); 2] = [
    ("Origin", "https://www.icloud.com"),
    ("Referer", "https://www.icloud.com/"),
];

static ACCOUNT_COUNTRY_HEADER: &str = "X-Apple-ID-Account-Country";
static SCNT_HEADER: &str = "scnt";
static SESSION_TOKEN_HEADER: &str = "X-Apple-Session-Token";
static SESSION_ID_HEADER: &str = "X-Apple-ID-Session-Id";
static TRUST_TOKEN_HEADER: &str = "X-Apple-TwoSV-Trust-Token";

static OAUTH_STATE_HEADER: &'static str = "X-Apple-OAuth-State";

static AUTH_ENDPOINT: &'static str = "https://idmsa.apple.com/appleauth/auth";
static SETUP_ENDPOINT: &'static str = "https://setup.icloud.com/setup/ws/1";

static APPLE_RESPONSE_HEADER: &str = "X-Apple-I-Rscd";

const AUTH_HEADERS: [(&'static str, &'static str); 7] = [
    (
        "X-Apple-OAuth-Client-Id",
        "d39ba9916b7251055b22c7f910e2ea796ee65e98b2ddecea8f5dde8d9d1a815d",
        ),
        ("X-Apple-OAuth-Client-Type", "firstPartyAuth"),
        ("X-Apple-OAuth-Redirect-URI", "https://www.icloud.com"),
        ("X-Apple-OAuth-Require-Grant-Code", "true"),
        ("X-Apple-OAuth-Response-Mode", "web_message"),
        ("X-Apple-OAuth-Response-Type", "code"),
        (
            "X-Apple-Widget-Key",
            "d39ba9916b7251055b22c7f910e2ea796ee65e98b2ddecea8f5dde8d9d1a815d",
            ),
];

#[derive(PartialEq)]
pub enum AuthenticationState {
    Unauthenticated,
    NeedsSecondFactor,
    Authenticated,
}

impl std::fmt::Display for AuthenticationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            AuthenticationState::Unauthenticated => {
                write!(f, "Unauthenticated")
            }
            AuthenticationState::NeedsSecondFactor => {
                write!(f, "Needs 2FA")
            }
            AuthenticationState::Authenticated => {
                write!(f, "Authenticated")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServiceInfo {
    pub url: String
}


#[derive(Serialize, Deserialize, Clone)]
pub struct SessionData {
    oauth_state: String,
    session_id: Option<String>,
    session_token: Option<String>,
    trust_token: Option<String>,
    scnt: Option<String>,
    account_country: Option<String>,
    cookies: BTreeSet<String>,
    webservices: BTreeMap<String, ServiceInfo>
}

impl SessionData {

    pub fn new() -> Result<SessionData, Error> {
       Ok(SessionData{
           oauth_state: format!("auth-{}", uuid::generate_uuid()?).to_string(),
           session_id: None,
           session_token: None,
           trust_token: None,
           scnt: None,
           account_country: None,
           cookies: BTreeSet::new(),
           webservices: BTreeMap::new()
       })
    }
}

pub struct Session {
    client: Client<HttpsConnector<HttpConnector>, Body>,
    data: SessionData,
    drive: Option<DriveService>
}

impl Session {
    pub fn new(mut data: SessionData) -> Result<Session, Error> {
        Ok(Session {
            client: Client::builder().build(
                        HttpsConnectorBuilder::new()
                        .with_native_roots()
                        .https_only()
                        .enable_http1()
                        .build(),
                        ),
            data: data,
            drive: None
        })
    }

    pub async fn request<F>(
        &mut self,
        method: Method,
        uri: String,
        body: Body,
        f: F,
        ) -> Result<Response<Body>, Error>
        where
        F: FnOnce(&mut http::request::Builder) -> Result<(), Error>,
        {
            let mut request_builder = Request::builder().method(method).uri(uri);

                request_builder =
                    request_builder.header(&String::from(OAUTH_STATE_HEADER), self.data.oauth_state.clone());

            if let Some(session_id) = &self.data.session_id {
                request_builder = request_builder.header(&String::from(SESSION_ID_HEADER), session_id);
            }

            if let Some(scnt) = &self.data.scnt {
                request_builder = request_builder.header(&String::from(SCNT_HEADER), scnt);
            }

            for (key, value) in GLOBAL_HEADERS {
                request_builder = request_builder.header(key, value);
            }


            if self.data.cookies.len() > 0 {
                let cookies: Vec<String> = self.data.cookies.iter().map(|x| x.clone()).collect();
                request_builder =
                    request_builder.header(hyper::header::COOKIE, cookies.as_slice().join("; "));
            }

            f(&mut request_builder)?;

            match self.client.request(request_builder.body(body)?).await {
                Ok(response) => {
                    if let Some(account_country) = response.headers().get(ACCOUNT_COUNTRY_HEADER) {
                        self.data.account_country = Some(String::from(account_country.to_str()?));
                    }

                    if let Some(session_id) = response.headers().get(SESSION_ID_HEADER) {
                        self.data.session_id = Some(String::from(session_id.to_str()?));
                    }

                    if let Some(session_token) = response.headers().get(SESSION_TOKEN_HEADER) {
                        self.data.session_token = Some(String::from(session_token.to_str()?));
                    }

                    if let Some(scnt) = response.headers().get(SCNT_HEADER) {
                        self.data.scnt = Some(String::from(scnt.to_str()?));
                    }

                    if let Some(trust_token) = response.headers().get(TRUST_TOKEN_HEADER) {
                        self.data.trust_token = Some(String::from(trust_token.to_str()?));
                    }

                    for (key, value) in response.headers() {
                        if key == hyper::header::SET_COOKIE {
                            if let Some(cookie) = value.to_str()?.split(";").next() {
                                self.data.cookies.insert(String::from(cookie));
                            }
                        }
                    }

                    match response.status() {
                        StatusCode::UNAUTHORIZED => {
                            Err(Error::AuthenticationFailed)
                        } _ => {
                            Ok(response)
                        }
                    }
                }
                Err(err) => Err(Error::from(err)),
            }
        }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Error> {
        let body = json!({
            "accountName" : username,
            "password" : password,
            "rememberMe": true,
            "trustTokens": []
        })
        .to_string();

        let uri = format!("{}/signin?isRememberMeEnable=true", AUTH_ENDPOINT);

        let response = self
            .request(Method::POST, uri, Body::from(body), |builder| {
                if let Some(headers) = builder.headers_mut() {
                    headers.insert("Content-Type", "application/json".parse()?);
                    headers.insert("Accept", "*/*".parse()?);
                    for (key, value) in AUTH_HEADERS {
                        headers.insert(key, value.parse()?);
                    }
                }
                Ok(())
            })
        .await?;

        if response.status() == StatusCode::OK {
            if let Some(rscd) = response.headers().get(APPLE_RESPONSE_HEADER) {
                if StatusCode::from_bytes(rscd.as_bytes())? != StatusCode::CONFLICT {
                    return Err(Error::InvalidCredentials);
                }
            }
            Ok(())
        } else {
            Err(Error::InvalidCredentials)
        }
    }

    pub async fn authenticate(&mut self) -> Result<AuthenticationState, Error> {
        let body = json!({
            "accountCountryCode": self.data.account_country.as_ref()
                .ok_or(Error::MissingCacheItem(String::from(ACCOUNT_COUNTRY_HEADER)))?,
                "dsWebAuthToken": self.data.session_token.as_ref()
                    .ok_or(Error::MissingCacheItem(String::from(SESSION_TOKEN_HEADER)))?.clone(),
                    "extended_login": true,
                    "trustToken": self.data.trust_token.as_ref().unwrap_or(&String::new())
        })
        .to_string();

        let uri = format!("{}/accountLogin", SETUP_ENDPOINT);

        let response = self
            .request(Method::POST, uri, Body::from(body), |builder| {
                if let Some(headers) = builder.headers_mut() {
                    headers.insert("Content-Type", "application/json".parse()?);
                    headers.insert("Accept", "*/*".parse()?);
                    for (key, value) in AUTH_HEADERS {
                        headers.insert(key, value.parse()?);
                    }
                }
                Ok(())
            })
        .await?;

        if response.status() == StatusCode::OK {
            let body = hyper::body::aggregate(response).await?;
            let auth_info: serde_json::Value = serde_json::from_reader(body.reader())?;

            if let Some(drivews_url) = auth_info["webservices"]["drivews"]["url"].as_str() {
                self.data.webservices.insert(String::from("drive"), ServiceInfo{
                    url: drivews_url.to_string()
                });
            }

            if auth_info["hsaChallengeRequired"] == true {
                if auth_info["hsaTrustedBrowser"] == true {
                    Ok(AuthenticationState::Authenticated)
                } else {
                    Ok(AuthenticationState::NeedsSecondFactor)
                }
            } else {
                Ok(AuthenticationState::Authenticated)
            }
        } else {
            Ok(AuthenticationState::Unauthenticated)
        }
    }

    pub async fn trust_session(&mut self) -> Result<(), Error> {
        let uri = format!("{}/2sv/trust", AUTH_ENDPOINT);

        let response = self
            .request(Method::GET, uri, Body::empty(), |builder| {
                if let Some(headers) = builder.headers_mut() {
                    headers.insert("Accept", "*/*".parse()?);
                    for (key, value) in AUTH_HEADERS {
                        headers.insert(key, value.parse()?);
                    }
                }
                Ok(())
            })
        .await?;

        if response.status() == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(Error::TrustFailed)
        }
    }

    pub async fn authenticate_2fa(&mut self, code: &str) -> Result<(), Error> {
        let uri = format!("{}/verify/trusteddevice/securitycode", AUTH_ENDPOINT);

        let body = json!({
            "securityCode": {
                "code": code
            }
        })
        .to_string();

        let response = self
            .request(Method::POST, uri, Body::from(body), |builder| {
                if let Some(headers) = builder.headers_mut() {
                    headers.insert("Content-Type", "application/json".parse()?);
                    headers.insert("Accept", "application/json".parse()?);
                    for (key, value) in AUTH_HEADERS {
                        headers.insert(key, value.parse()?);
                    }
                }
                Ok(())
            })
        .await?;

        if response.status() == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(Error::AuthenticationFailed)
        }
    }

    pub fn get_service_info(&self, name: String) -> Option<&ServiceInfo> {
        self.data.webservices.get(&name)
    }

    pub fn data(&self) -> &SessionData {
        &self.data
    }
}
