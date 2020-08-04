use std::sync::{RwLock, RwLockReadGuard};
use std::collections::HashMap;
use std::time::SystemTime;
use rand::Rng;
use crate::auth::UserID;

fn get_rand_token<const N: usize>() -> [u8; N] {
    let mut res = [0; N];

    let mut valid_chars = ('a'..='z').chain('A'..='Z').chain('0'..='9').cycle();
    let mut rng = rand::thread_rng();
    for i in 0..N {
        res[i] = valid_chars.nth(rng.gen::<usize>() % 100).unwrap() as u8;
    }
    res
}

type Token = [u8; crate::auth::AUTH_TOKEN_LEN];

pub struct ActiveTokenStorage {
    user_tokens: RwLock<HashMap<Token, (SystemTime, UserID)>>
}


impl ActiveTokenStorage {
    pub fn empty() -> Self {
        ActiveTokenStorage {
            user_tokens: RwLock::new(HashMap::new())
        }
    }

    pub fn inner(&self) -> RwLockReadGuard<HashMap<Token, (SystemTime, UserID)>> {
        self.user_tokens.read().unwrap()
    }

    pub fn with_debug_access_token() -> Self {
        let mut hm = HashMap::new();
        let dat: Vec<u8> = "0123456789abcdef".chars().map(|c| c as u8).collect();
        assert_eq!(dat.len(), 16);
        unsafe {
            hm.insert(std::mem::transmute_copy(&*dat.as_ptr()), (SystemTime::now(), UserID::debug_access()));
        }

        ActiveTokenStorage {
            user_tokens: RwLock::new(hm)
        }
    }

    pub fn get_user_data(&self, token: &[u8]) -> Option<
    (SystemTime, UserID)> {
        self.user_tokens.read().ok().map(|hm| hm.get(token).cloned()).flatten()
    }

    pub fn new_user_token(&self, user_id: UserID) -> Token {

        let token = get_rand_token();
        self.user_tokens.write().unwrap().insert(token, (SystemTime::now(), user_id));
        token
    }
}