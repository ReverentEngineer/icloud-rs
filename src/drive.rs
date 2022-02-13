use std::sync::Arc;
use futures::lock::Mutex;
use crate::error::Error;
use crate::session::Session;
use chrono::{DateTime, FixedOffset};
use hyper::body::Buf;
use hyper::{Body, Method, StatusCode};
use serde_json::json;
use serde_json::value::Value;

#[derive(Clone)]
pub struct File {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub date_created: DateTime<FixedOffset>,
    pub date_changed: DateTime<FixedOffset>,
    pub date_modified: DateTime<FixedOffset>,
    pub last_opened: Option<DateTime<FixedOffset>>,
}

#[derive(Clone)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub date_created: DateTime<FixedOffset>,
    pub items: Vec<DriveNode>,
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

#[derive(Clone)]
pub enum DriveNode {
    Folder(Folder),
    File(File)
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
            })),
            "FILE" => Ok(DriveNode::File(File {
                id: String::from(value["drivewsid"].as_str().unwrap()),
                name: String::from(value["name"].as_str().unwrap()),
                size: value["size"].as_u64().unwrap(),
                date_created: DateTime::parse_from_rfc3339(value["dateCreated"].as_str().unwrap())?,
                date_changed: DateTime::parse_from_rfc3339(value["dateChanged"].as_str().unwrap())?,
                date_modified: DateTime::parse_from_rfc3339(value["dateModified"].as_str().unwrap())?,
                last_opened: value["lastOpenTime"].as_str().map_or(None, |time| Some(DateTime::parse_from_rfc3339(time).ok()?)),
            })),
            _ => {
                Err(Error::InvalidDriveNodeType)
            }
        }
    }

    pub fn id(&self) -> &String {
        match self {
            DriveNode::Folder(folder) => &folder.id,
            DriveNode::File(file) => &file.id,
        }
    }

    pub fn name(&self) -> &String {
        match self {
            DriveNode::Folder(folder) => &folder.name,
            DriveNode::File(file) => &file.name,
        }
    }

    pub fn date_created(&self) -> DateTime<FixedOffset> {
        match self {
            DriveNode::Folder(folder) => folder.date_created,
            DriveNode::File(file) => file.date_created,
        }
    }
}

impl std::fmt::Display for DriveNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            DriveNode::Folder(folder) => {
                write!(
                    f,
                    "Folder(id={},name={},dateCreated={},items={})",
                    folder.id,
                    folder.name,
                    folder.date_created,
                    folder.items.len()
                )
            },
            DriveNode::File(file) => {
                write!(
                    f,
                    "File(id={},name={},dateCreated={},size={})",
                    file.id,
                    file.name,
                    file.date_created,
                    file.size,
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

    pub async fn root(&mut self) -> Result<Folder, Error> {
        match self.get_node("FOLDER::com.apple.CloudDocs::root").await? {
            DriveNode::Folder(folder) => Ok(folder),
            _ => Err(Error::InvalidDriveNodeType)
        }
    }

    pub async fn get_node(&mut self, id: &str) -> Result<DriveNode, Error> {
        let uri = format!("{}/retrieveItemDetailsInFolders", self.url);
        let body = json!([
                         {
                             "drivewsid": id,
                             "partialData": false
                         }
        ])
        .to_string();

        let mut session = self.session.lock().await;
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
                Err(Error::AuthenticationFailed(String::from("Failed to authenticate to Drive")))
            }
    }
}
