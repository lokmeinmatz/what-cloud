use std::path::{Path, PathBuf};
use std::sync::Once;
use log::{info, warn};
use std::collections::HashMap;


#[derive(Deserialize, Debug, Clone)]
pub struct IconConf {
    #[serde(rename = "displayText")]
    pub display_text: String,
    pub color: String
}


struct ConfigStore {
    data_path: PathBuf,
    db_path: PathBuf,
    icon_conf: HashMap<String, IconConf>
}


static mut CONFIG_STORE: Option<ConfigStore> = None;
static INIT_ONCE: Once = Once::new();

/// Conly call after init()!
pub fn debug_config_store() -> String {
    let mut res = String::with_capacity(256);

    res.push_str("ConfigStore:\n");
    res.push_str("\tdata_path: ");
    res.push_str(data_path().to_string_lossy().as_ref());
    res.push_str("\n\tdb_path: ");
    res.push_str(db_path().to_string_lossy().as_ref());
    res.push_str(&format!("\n\tStored icon confs: {}", icon_confs().len()));
    res
}

pub fn init() -> Result<(), &'static str> {
    INIT_ONCE.call_once(|| {
        info!("Initializing ConfigStore...");

        let m_data_path = std::env::var("DATA_PATH");
        if m_data_path.is_err() {
            warn!("DATA_PATH not set, setting to default...");
        }

        let m_db_path = std::env::var("DB_PATH");
        
        if m_db_path.is_err() {
            warn!("DB_PATH not set, setting to default...");
        }
        let icon_conf_path = std::env::var("ICON_CONF").unwrap_or("./icon-conf.json".into());
        
        // load icon conf
        let mut icon_conf = HashMap::new();

        match std::fs::read_to_string(&icon_conf_path) {
            Ok(json) => {
                match serde_json::from_str(&json) {
                    Ok(p) => icon_conf = p,
                    Err(e) => warn!("Error while parsing {}, ignoring file. {:?}", icon_conf_path, e)
                }
            },
            Err(e) => {
                warn!("Failed to load {}, no settings loaded! {:?}", icon_conf_path, e);
            }
        }


        let conf = ConfigStore {
            data_path: PathBuf::from(m_data_path.unwrap_or("./test_data".into())),
            db_path: PathBuf::from(m_db_path.unwrap_or("./database.sqlite".into())),
            icon_conf
        };
        unsafe {
            assert!(CONFIG_STORE.is_none());
            CONFIG_STORE = Some(conf);
        }



        info!("ConfigStore init finished\n{}", debug_config_store());
    });

    Ok(())
}

unsafe fn conf() -> &'static ConfigStore {
    CONFIG_STORE.as_ref().expect("Config not initialized")
}

pub fn icon_confs() -> &'static HashMap<String, IconConf> {
    unsafe {
        &conf().icon_conf
    }
}

pub fn data_path() -> &'static Path {
    unsafe {
        conf().data_path.as_path()
    }
}

pub fn db_path() -> &'static Path {
    unsafe {
        conf().db_path.as_path()
    }
}


