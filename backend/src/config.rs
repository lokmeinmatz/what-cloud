use std::cell::UnsafeCell;
use std::sync::Once;

struct ConfigStore {
    data_base_path: String
}


static mut CONFIG_STORE: Option<ConfigStore> = None;
static INIT_ONCE: Once = Once::new();

pub fn init() -> Result<(), &'static str> {
    INIT_ONCE.call_once(|| {
        let conf = ConfigStore {
            data_base_path: std::env::var("DATA_BASE_PATH").expect("Set DATA_BASE_PATH to point \
            to where the user data is stored."),
        };
        unsafe {
            assert!(CONFIG_STORE.is_none());
            CONFIG_STORE = Some(conf);
        }
    });

    Ok(())
}

pub fn data_base_path() -> &'static str {
    unsafe {
        CONFIG_STORE.as_ref().expect("Config not initialized").data_base_path.as_str()
    }
}