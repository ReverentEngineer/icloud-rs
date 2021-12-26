use crate::error::Error;
use crate::session::Session;
use chrono::{DateTime, Local};
use hyper::body::Buf;
use hyper::{Body, Method, StatusCode};
use serde_json::json;
use std::sync::{Arc, Mutex};

pub enum DriveNodeType {
    Folder,
}

impl TryFrom<String> for DriveNodeType {
    type Error = Error;

    fn try_from(s: String) -> Result<DriveNodeType, Error> {
        match s.as_str() {
            "FOLDER" => Ok(DriveNodeType::Folder),
            _ => Err(Error::AuthenticationFailed),
        }
    }
}

pub struct DriveNode {
    id: String,
    name: String,
    r#type: DriveNodeType,
    size: u64,
    date_changed: DateTime<Local>,
    date_modified: DateTime<Local>,
    date_last_opened: DateTime<Local>,
    items: Vec<DriveNode>,
}

impl std::fmt::Display for DriveNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "DriveNode(id={},name={},items={}(",
            self.id,
            self.name,
            self.items.len()
            )
    }
}

pub struct DriveService {
    session: Arc<Mutex<Session>>,
    url: String,
}

impl DriveService {
    pub fn new(session: Arc<Mutex<Session>>, url: String) -> DriveService {
        DriveService {
            session: session,
            url: url,
        }
    }

    pub async fn root(&mut self) -> Result<DriveNode, Error> {
        self.get_node_data("root").await
    }

    async fn get_node_data(&mut self, id: &str) -> Result<DriveNode, Error> {
        let uri = format!("{}/retrieveItemDetailsInFolders", self.url);
        let body = json!([
                         {
                             "drivewsid": format!("FOLDER::com.apple.CloudDocs::{}", id),
                             "partialData": false
                         }
        ])
            .to_string();

        if let Ok(mut session) = self.session.lock() { 
            let response = session
                .request(Method::POST, uri, Body::from(body), |builder| {
                    if let Some(headers) = builder.headers_mut() {
                        headers.insert("Content-Type", "application/json".parse()?);
                        headers.insert("Accept", "application/json".parse()?);
                    }
                    Ok(())
                })
            .await?;


            if response.status() == StatusCode::OK {
                let body = hyper::body::aggregate(response).await?;
                let drive_node: serde_json::Value = serde_json::from_reader(body.reader())?;
                println!("{}", drive_node[0]["items"][1]);
                Ok(DriveNode {
                    id: String::from(drive_node[0]["drivewsid"].as_str().unwrap()),
                    name: String::from(drive_node[0]["name"].as_str().unwrap()),
                    r#type: DriveNodeType::try_from(String::from(
                            drive_node[0]["type"].as_str().unwrap(),
                            ))?,
                            size: 0,
                            date_changed: Local::now(),
                            date_modified: Local::now(),
                            date_last_opened: Local::now(),
                            items: Vec::new(),
                })
            } else {
                let body = hyper::body::aggregate(response).await?;
                let drive_info: serde_json::Value = serde_json::from_reader(body.reader())?;
                println!("{}", drive_info);
                Err(Error::AuthenticationFailed)
            }
        } else {
            Err(Error::MutexError)
        }
    }
}
