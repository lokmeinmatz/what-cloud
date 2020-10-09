use rusqlite::{Connection, params, Row, ToSql};
use std::sync::{Mutex, MutexGuard};
use log::{info, trace, warn};
use std::convert::{TryFrom, TryInto};
use std::path::{Path, PathBuf};
use crate::auth::UserID;
use crate::fs::shared::{SharedEntry, SharedID};

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

    pub fn get_share_id(&self, user_id: &UserID, path: &std::path::Path) -> Result<Option<String>, ()> {
        use rusqlite::OptionalExtension;
        let conn = self.conn();
        conn.query_row(
            "SELECT ID FROM SHARED WHERE USER = ? AND BASE_PATH = ?",
            params![&user_id.0, path.to_str().unwrap()],
        |row| row.get(0)).optional().map_err(|_| ())
        
    }


    pub fn get_shared_entry(&self, maybe_shared_id: &str) -> Option<SharedEntry> {
        let conn = self.conn();
        // needs rusqlite::Error as E type because get()? returns Err(rusqlite::Error) even though it is never used
        conn.query_row_and_then::<_,rusqlite::Error,_,_>("SELECT ID, USER, BASE_PATH FROM SHARED WHERE ID = ?", &[maybe_shared_id], |row| {
            let id = SharedID::from_string_unchecked(row.get(0)?);
            let user = UserID(row.get(1)?);
            let path = row.get::<_, String>(2)?.into();
            Ok(SharedEntry {
                path,
                share_id: id,
                user
            })
        }).ok()
    }

    #[allow(dead_code)]
    pub fn is_active_shared_id(&self, id: &str) -> bool {
        let conn = self.conn();
        // returns 1 if id exists in table
        conn.query_row::<u32, _, _>("SELECT EXISTS(SELECT ID FROM SHARED WHERE ID = ?)", &[id], |r| r.get(0)).unwrap() == 1
    }

    pub fn get_all_shared(&self, user_id: &UserID) -> Vec<crate::fs::shared::SharedEntry> {
        let conn = self.conn();
        let mut prep = conn.prepare("SELECT ID, BASE_PATH FROM SHARED WHERE USER = ?").unwrap();

        prep.query_map(
            params![&user_id.0], 
            |r| {Ok((r.get(0), r.get::<_, String>(1)))}).unwrap()
        .filter_map(|r| {
            match r {
                Ok((Ok(id), Ok(path))) => Some(crate::fs::shared::SharedEntry {
                    path: PathBuf::from(path),
                    user: user_id.clone(),
                    share_id: SharedID::from_string_unchecked(id)
                }),
                _ => None
            }
        }).collect()

    }



    /// if enabled, returns the share id
    /// expects path to be valid 
    /// `upload_limit` in megabytes
    pub fn update_share(&self, user_id: &UserID, path: &std::path::Path, enabled: bool, mut upload_limit: Option<u32>) -> Option<SharedID> {
        let conn = self.conn();
        let path_str = path.to_str().unwrap();
        if enabled {

            // first check if share allready exists
            let id: SharedID = match conn.query_row(
                "SELECT ID FROM SHARED WHERE USER = ? AND BASE_PATH = ?",
                params![&user_id.0, path_str],
            |row| row.get(0).map(|s| SharedID::from_string_unchecked(s))).ok() {
                Some(id) => id,
                None => {
                    // generate new unique id
                    loop {
                        let share_id: String = crate::token_validizer::get_rand_token::<16>().iter().map(|e| *e as char).collect(); 
                        if let Ok(false) = conn.query_row(
                            "SELECT EXISITS(SELECT 1 FROM SHARED WHERE ID = ?)", 
                            &[&share_id], |r| r.get::<usize, u32>(0).map(|e| e == 1)) {
                            // shared id doesn't exist
                            break SharedID::from_string_unchecked(share_id)
                        } else {
                            warn!("Generated existing shared id, retry...");
                        }
                    }
                }
            };

            if let Some(0) = upload_limit {
                upload_limit = None;
            }

            // very unperformant
            let upload_limit: Box<dyn ToSql> = upload_limit.map(|ul| Box::new(ul) as Box<dyn ToSql>).unwrap_or_else(|| Box::new(rusqlite::types::Null));


            // no share exists, create new
            // TODO check if share id allready exisits?

            conn.execute(
                "INSERT OR REPLACE INTO SHARED (ID, USER, BASE_PATH, CREATED_AT, UPLOAD_LIMIT) VALUES (?, ?, ?, datetime('now'))", 
                params![id.as_ref(), &user_id.0, path_str, upload_limit]).ok()?;
            Some(id)
        } else {

            conn.execute("DELETE FROM SHARED WHERE BASE_PATH = ?", params![path_str]).ok();
            None
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

#[allow(dead_code)]
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

