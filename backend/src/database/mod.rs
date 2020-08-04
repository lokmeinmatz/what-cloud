use rusqlite::{Connection, params, Row};
use std::sync::{Mutex, MutexGuard};
use log::{info, trace};
use std::convert::{TryFrom, TryInto};
use std::path::Path;
use crate::auth::UserID;

pub struct SharedDatabase {
    conn: Mutex<Connection>
}

impl SharedDatabase {
    pub fn new(path: &Path) -> Self {
        info!("Opening database {:?}", path);
        SharedDatabase {
            conn: Mutex::new(Connection::open(path).expect("Failed to open database"))
        }
    }

    fn conn(&self) -> MutexGuard<Connection> {
        // if this fails how was it instantiated???
        self.conn.lock().unwrap()
    }

    pub fn get_user(&self, query: GetUserQuery) -> Option<DBUser> {
        let conn = self.conn();
        match query {
            GetUserQuery::ByName(name) => {
                let mut stmt = conn.prepare(
                    "SELECT ID, NAME, PASSWORD_HASH FROM USERS WHERE NAME = ?").unwrap();
                let n = stmt.query_map(params![name], user_from_row).unwrap().next();
                println!("db user by name {} -> {:?}", name, n);
                return n.map(|e| {
                    //eprintln!("{:?}", e);
                    e.ok()
                }).flatten();
            },
            _ => None
        }
    }
}


impl TryFrom<String> for UserID {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != 8 {
            return Err(())
        }
        Ok(UserID(value))
    }
}

pub enum GetUserQuery<'a> {
    ByName(&'a str),
    ByID(&'a UserID)
}

#[derive(Debug)]
pub enum UserRoll {
    Guest = 0,
    User = 1,
    Admin = 2
}

impl TryFrom<u32> for UserRoll {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(UserRoll::Guest),
            1 => Ok(UserRoll::User),
            2 => Ok(UserRoll::Admin),
            _ => Err(())
        }
    }
}

#[derive(Debug)]
pub struct DBUser {
    pub name: String,
    pub id: UserID,
    pub hashed_pw: String,
    pub roll: UserRoll
}

fn user_from_row(row: &Row) -> Result<DBUser, rusqlite::Error> {
    let id = row.get::<usize, String>(0)?.try_into().map_err(|_| rusqlite::Error::InvalidQuery)?;
    let name = row.get(1)?;
    let hashed_pw = row.get(2)?;
    let roll: UserRoll = row.get::<usize, u32>(3).map(UserRoll::try_from).map(Result::ok).ok()
        .flatten()
        .unwrap_or
    (UserRoll::Guest);
    trace!("user from row: {:?} {} {} {:?}", id, name, hashed_pw, roll);
    Ok(DBUser {
        id, name, hashed_pw, roll
    })
}

