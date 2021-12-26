use std::path::Path;
use std::io::{
    stdin, 
    stdout,
    BufReader,
    BufWriter,
    Write
};
use std::fs::File;

extern crate icloud;
use crate::icloud::error::Error;
use crate::icloud::session::{
    AuthenticationState, 
    SessionData
};
use crate::icloud::Client;


async fn login_prompt() -> (String, String) {
    print!("Enter username: ");
    stdout().flush().unwrap();
    let mut username = String::new();
    if let Err(msg) = stdin().read_line(&mut username) {
        panic!("{}", msg);
    }
    username.truncate(username.len() - 1);
    print!("Enter password: ");
    stdout().flush().unwrap();
    let mut password = String::new();
    if let Err(msg) = stdin().read_line(&mut password) {
        panic!("{}", msg);
    }
    password.truncate(password.len() - 1);
    (username, password)
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {

    let path = Path::new("cache.json");
    let session_data : SessionData = if path.exists() {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)?
    } else {
        SessionData::new()?
    };

    if let Ok(mut icloud) = Client::new(session_data) {
        if let Some(mut drive) = icloud.drive() {
            drive.root().await?;
        }
    
        let file = if path.exists() {
            File::open(path)
        } else {
            File::create(path)
        }?;
        let writer = BufWriter::new(file);
        let data = icloud.save();
        serde_json::to_writer(writer, &data)?;
    }

    Ok(())
}
