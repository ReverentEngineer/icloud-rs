use std::sync::{
    Arc,
    Mutex
};

use crate::drive::DriveService;
use crate::error::Error;
use crate::session::{
    Session,
    SessionData
};

pub struct Client {
    session: Arc<Mutex<Session>>
}

impl Client {

    pub fn new(data: SessionData) -> Result<Client, Error> {
        Ok(Client{
            session: Arc::new(Mutex::new(Session::new(data)?))
        })
    }

    pub fn drive(&mut self) -> Option<DriveService> {
        let clone = self.session.clone();
        self.session.lock().ok().map_or(None, |session| {
            session.get_service_info(String::from("drive")).map(|s| {
                DriveService::new(clone, s.url.clone())
            })
        })
    }

    pub fn save(&mut self) -> Option<SessionData> {
        self.session.lock().ok().map_or(None, |s| Some(s.data().clone()))
    }
}
