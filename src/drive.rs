use crate::error::Error;
use crate::session::Session;
use chrono::{DateTime, FixedOffset};
use hyper::body::Buf;
use hyper::{Body, Method, StatusCode};
use serde_json::json;
use serde_json::value::Value;
use std::sync::{Arc, Mutex};

pub struct Folder {
    id: String,
    name: String,
    date_created: DateTime<FixedOffset>,
    items: Vec<DriveNode>,
}

pub struct FolderIter<'a> {
    current: std::slice::Iter<'a, DriveNode>
}

impl<'a> Iterator for FolderIter<'a> {

    type Item = &'a DriveNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.next()
    }

}

impl Folder {

    pub fn iter(&self) -> FolderIter {
        FolderIter{
            current: self.items.iter()
        }
    }

}

pub enum DriveNode {
    Folder(Folder),
}

impl DriveNode {
    fn new(value: &Value) -> Result<DriveNode, Error> {
        match value["type"].as_str().ok_or(Error::InvalidDriveNodeType)? {
            "FOLDER" => Ok(DriveNode::Folder(Folder {
                id: String::from(value["drivewsid"].as_str().unwrap()),
                name: String::from(value["name"].as_str().unwrap()),
                date_created: DateTime::parse_from_rfc3339(value["dateCreated"].as_str().unwrap())?,
                items: value["items"].as_array().map_or(Vec::new(), |array| {
                    let mut items = Vec::new();
                    for item in array {
                        if let Ok(node) = DriveNode::new(item) {
                            items.push(node);
                        }
                    }
                    items
                }),
            })), _ => {
                Err(Error::InvalidDriveNodeType)
            }
        }
    }

    pub fn id(&self) -> &String {
        match self {
            DriveNode::Folder(folder) => &folder.id,
        }
    }

    pub fn name(&self) -> &String {
        match self {
            DriveNode::Folder(folder) => &folder.name,
        }
    }

    pub fn date_created(&self) -> DateTime<FixedOffset> {
        match self {
            DriveNode::Folder(folder) => folder.date_created,
        }
    }
}

impl std::fmt::Display for DriveNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            DriveNode::Folder(folder) => {
                write!(
                    f,
                    "Folder(id={},name={},dateCreated={}, items={})",
                    folder.id,
                    folder.name,
                    folder.date_created,
                    folder.items.len()
                )
            }
        }
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
                Ok(DriveNode::new(&drive_node[0])?)
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
