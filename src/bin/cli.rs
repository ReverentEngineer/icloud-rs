use std::fs::File;
use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use std::path::Path;

extern crate icloud;
use crate::icloud::drive::DriveNode;
use crate::icloud::error::Error;
use crate::icloud::SessionData;
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

async fn prompt_2fa() -> String {
    print!("Enter 2FA code: ");
    stdout().flush().unwrap();
    let mut code = String::new();
    if let Err(msg) = stdin().read_line(&mut code) {
        panic!("{}", msg);
    }
    code.truncate(code.len() - 1);
    code
}

async fn authenticate(client: &mut Client) -> Result<(), Error> {
    match client.authenticate().await {
        Err(Error::AuthenticationFailed(_)) | Err(Error::MissingCacheItem(_)) => {
            let (username, password) = login_prompt().await;
            match client.login(username.as_str(), password.as_str()).await {
                Ok(()) => Ok(()),
                Err(err) => match err {
                    Error::Needs2FA => {
                        let code = prompt_2fa().await;
                        client.authenticate_2fa(code.as_str()).await?;
                        Ok(())
                    }
                    _ => Err(err),
                },
            }
        }
        Err(Error::Needs2FA) => {
            let code = prompt_2fa().await;
            client.authenticate_2fa(code.as_str()).await?;
            Ok(())
        }
        Err(err) => {
            println!("{}", err);
            Err(err)
        }
        Ok(()) => Ok(()),
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let path = Path::new("cache.json");
    let session_data: SessionData = if path.exists() {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)?
    } else {
        SessionData::new()?
    };

    if let Ok(mut client) = Client::new(session_data) {
        authenticate(&mut client).await?;

        if let Some(mut drive) = client.drive().await {
            let root = drive.root().await?;
            for item in root.iter() {
                let item = drive.get_node(item.id()).await?;
                println!("{}", item);
                match item {
                    DriveNode::Folder(folder) => {
                        for item in folder.iter() {
                            println!("{}", item);
                        }
                    }, _ => {

                    }
                }
            }
        }

        let file = if path.exists() {
            File::options().write(true).open(path)
        } else {
            File::create(path)
        }?;
        let writer = BufWriter::new(file);
        let data = client.save().await;
        serde_json::to_writer(writer, &data)?;
    }

    Ok(())
}
