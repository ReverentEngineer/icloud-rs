use crate::drive::DriveService;
use crate::error::Error;
use crate::session::{Session, SessionData};
use std::sync::Arc;
use futures::lock::Mutex;

pub struct Client {
    session: Arc<Mutex<Session>>,
}

impl Client {
    pub fn new(data: SessionData) -> Result<Client, Error> {
        Ok(Client {
            session: Arc::new(Mutex::new(Session::new(data)?)),
        })
    }

    pub async fn drive(&mut self) -> Option<DriveService> {
        let clone = self.session.clone();
        let session = self.session.lock().await;
        session
            .get_service_info(String::from("drive"))
            .map(|s| DriveService::new(clone, s.url.clone()))
    }

    pub async fn authenticate(&mut self) -> Result<(), Error> {
        let mut session = self.session.lock().await;
        session.authenticate().await
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Error> {
        let mut session = self.session.lock().await;
        session.login(username, password).await
    }

    pub async fn authenticate_2fa(&mut self, code: &str) -> Result<(), Error> {
        let mut session = self.session.lock().await;
        session.authenticate_2fa(code).await
    }

    pub async fn save(&mut self) -> Option<SessionData> {
        let session = self.session.lock().await;
        Some(session.data().clone())
    }
}
