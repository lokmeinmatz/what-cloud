use crate::auth::UserID;
use crate::fs::shared::{SharedEntry, SharedID};
use log::{error, info, trace, warn};
use rusqlite::{params, Connection, Result, Row, ToSql};
use std::convert::{TryFrom, TryInto};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

pub struct SharedDatabase {
    conn: Mutex<Connection>,
}

impl SharedDatabase {
    pub fn new(path: &Path) -> Self {
        info!("Opening database {:?}", path);
        SharedDatabase {
            conn: Mutex::new(Connection::open(path).expect("Failed to open database")),
        }
    }

    fn conn(&self) -> MutexGuard<Connection> {
        // if this fails how was it instantiated???
        self.conn.lock().unwrap()
    }

    pub fn get_user(&self, query: GetUserQuery) -> rusqlite::Result<DBUser> {
        let conn = self.conn();
        match query {
            GetUserQuery::ByName(name) => {
                let mut stmt = conn
                    .prepare("SELECT ID, NAME, PASSWORD_HASH, ROLLS FROM USERS WHERE NAME = ?")?;
                let n = stmt.query_map(params![name], user_from_row)?.next();
                println!("db user by name {} -> {:?}", name, n);
                return match n {
                    Some(r) => r,
                    None => Err(rusqlite::Error::QueryReturnedNoRows)
                }
            }
            _ => panic!("get_user branch not implemented"),
        }
    }

    pub fn get_all_users(&self) -> rusqlite::Result<Vec<DBUser>> {
        let conn =self.conn();
        let mut stmt = conn
            .prepare("SELECT ID, NAME, PASSWORD_HASH,ROLLS FROM USERS")
            .unwrap();
        let n = stmt.query_map(params![], user_from_row).unwrap();
        return Ok(n.filter_map(|res| {
            if let Err(e) = &res {
                warn!("{:?}", e);
            }
            res.ok()
        }).collect());
    }

    pub fn get_share_id(
        &self,
        user_id: &UserID,
        path: &std::path::Path,
    ) -> Result<Option<String>, ()> {
        use rusqlite::OptionalExtension;
        let conn = self.conn();
        conn.query_row(
            "SELECT ID FROM SHARED WHERE USER = ? AND BASE_PATH = ?",
            params![&user_id.0, path.to_str().unwrap()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| ())
    }

    pub fn get_shared_entry(&self, maybe_shared_id: &str) -> Option<SharedEntry> {
        let conn = self.conn();
        // needs rusqlite::Error as E type because get()? returns Err(rusqlite::Error) even though it is never used
        conn.query_row_and_then::<_, rusqlite::Error, _, _>(
            "SELECT ID, USER, BASE_PATH FROM SHARED WHERE ID = ?",
            &[maybe_shared_id],
            |row| {
                let id = SharedID::from_string_unchecked(row.get(0)?);
                let user = UserID(row.get(1)?);
                let path = row.get::<_, String>(2)?.into();
                Ok(SharedEntry {
                    path,
                    share_id: id,
                    user,
                })
            },
        )
        .ok()
    }

    #[allow(dead_code)]
    pub fn is_active_shared_id(&self, id: &str) -> bool {
        let conn = self.conn();
        // returns 1 if id exists in table
        conn.query_row::<u32, _, _>(
            "SELECT EXISTS(SELECT ID FROM SHARED WHERE ID = ?)",
            &[id],
            |r| r.get(0),
        )
        .unwrap()
            == 1
    }

    pub fn get_all_shared(&self, user_id: &UserID) -> Vec<crate::fs::shared::SharedEntry> {
        let conn = self.conn();
        let mut prep = conn
            .prepare("SELECT ID, BASE_PATH FROM SHARED WHERE USER = ?")
            .unwrap();

        prep.query_map(params![&user_id.0], |r| {
            Ok((r.get(0), r.get::<_, String>(1)))
        })
        .unwrap()
        .filter_map(|r| match r {
            Ok((Ok(id), Ok(path))) => Some(crate::fs::shared::SharedEntry {
                path: PathBuf::from(path),
                user: user_id.clone(),
                share_id: SharedID::from_string_unchecked(id),
            }),
            _ => None,
        })
        .collect()
    }

    /// if enabled, returns the share id
    /// expects path to be valid
    /// `upload_limit` in megabytes
    pub fn update_share(
        &self,
        user_id: &UserID,
        path: &std::path::Path,
        enabled: bool,
        mut upload_limit: Option<u32>,
    ) -> Option<SharedID> {
        let conn = self.conn();
        let path_str = path.to_str().unwrap();
        let mut shared_id: Option<SharedID> = None;

        if enabled {
            // first check if share allready exists
            shared_id = match conn
                .query_row(
                    "SELECT ID FROM SHARED WHERE USER = ? AND BASE_PATH = ?",
                    params![&user_id.0, path_str],
                    |row| row.get(0).map(|s| SharedID::from_string_unchecked(s)),
                )
                .ok()
            {
                Some(id) => Some(id),
                None => {
                    // generate new unique id
                    let mut id = None;
                    for _ in 0..100 {
                        let share_id: String = crate::utils::get_rand_token::<16>()
                            .iter()
                            .map(|e| *e as char)
                            .collect();
                        println!("Generated share id {}", share_id);
                        match conn.query_row(
                            "SELECT EXISTS(SELECT 1 FROM SHARED WHERE ID = ?)",
                            &[&share_id],
                            |r| r.get::<usize, u32>(0).map(|e| e == 1),
                        ) {
                            Ok(false) => {
                                // shared id doesn't exist
                                id = Some(SharedID::from_string_unchecked(share_id));
                                break;
                            }
                            Ok(true) => warn!("Generated existing shared id, retry..."),
                            Err(e) => {
                                error!("{:?}", e);
                                break;
                            }
                        }
                    }
                    id
                }
            };

            if let Some(0) = upload_limit {
                upload_limit = None;
            }

            dbg!(&shared_id);

            // no share exists, create new
            // TODO check if share id allready exisits?
        }
        match shared_id {
            Some(id) => {
                // very unperformant
                let upload_limit: Box<dyn ToSql> = upload_limit
                    .map(|ul| Box::new(ul) as Box<dyn ToSql>)
                    .unwrap_or_else(|| Box::new(rusqlite::types::Null));
                conn.execute(
                    "INSERT OR REPLACE INTO SHARED (ID, USER, BASE_PATH, CREATED_AT, UPLOAD_LIMIT) VALUES (?, ?, ?, datetime('now'), ?)", 
                    params![id.as_ref(), &user_id.0, path_str, upload_limit]).map_err(|e| error!("{:?}", e)).ok()?;
                Some(id)
            }
            None => {
                conn.execute("DELETE FROM SHARED WHERE BASE_PATH = ?", params![path_str])
                    .ok();
                None
            }
        }
    }
}

impl TryFrom<String> for UserID {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != 8 {
            return Err(());
        }
        Ok(UserID(value))
    }
}

#[allow(dead_code)]
pub enum GetUserQuery<'a> {
    ByName(&'a str),
    ByID(&'a UserID),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum UserRoll {
    Guest = 0,
    User = 1,
    Admin = 2,
}

impl TryFrom<u32> for UserRoll {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(UserRoll::Guest),
            1 => Ok(UserRoll::User),
            2 => Ok(UserRoll::Admin),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct DBUser {
    pub name: String,
    pub id: UserID,
    pub hashed_pw: String,
    pub roll: UserRoll,
}

fn user_from_row(row: &Row) -> Result<DBUser, rusqlite::Error> {
    let id = row
        .get::<usize, String>(0)?
        .try_into()
        .map_err(|_| rusqlite::Error::InvalidQuery)?;
    let name = row.get(1)?;
    let hashed_pw = row.get(2)?;
    let roll: UserRoll = row
        .get::<usize, u32>(3)
        .map(UserRoll::try_from)
        .map(Result::ok)
        .ok()
        .flatten()
        .unwrap_or(UserRoll::Guest);
    trace!("user from row: {:?} {} {} {:?}", id, name, hashed_pw, roll);
    Ok(DBUser {
        id,
        name,
        hashed_pw,
        roll,
    })
}
