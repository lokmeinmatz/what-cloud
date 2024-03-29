use log::{info, warn};
use rocket::http;
use rocket::response;
use rocket::State;
use std::collections::VecDeque;
use std::sync::RwLock;

const MAX_ICONS_CACHED: usize = 128;

/// LRU storage (ext, svg)
pub struct IconsCache(RwLock<VecDeque<(String, String)>>);

impl IconsCache {
    pub fn empty() -> Self {
        IconsCache(RwLock::new(VecDeque::new()))
    }

    pub fn get(&self, ext: &str) -> Option<String> {
        // load weak, upgrade, and clone the pointed to string
        self.0.read().ok().and_then(|hm| {
            hm.iter()
                .find(|entry| entry.0 == ext)
                .map(|(_, svg)| svg.clone())
        })
    }

    pub fn insert_new(&self, ext: String, svg: String) {
        // move svg into arc
        let mut w_cache = self.0.write().unwrap();

        // add into lookup
        w_cache.push_front((ext, svg));

        if w_cache.len() > MAX_ICONS_CACHED {
            let (ext, _) = w_cache.pop_front().unwrap();
            info!("Icon cache filled, removed {}", ext);
        }
    }
}

use response::content;
/// Sends token on success, else error
#[get("/static/icons/<ext>")]
pub fn icons_get(
    mut ext: String,
    cache: &State<IconsCache>,
) -> Result<content::Custom<String>, rocket::response::status::NotFound<()>> {


    if ext.ends_with(".svg") {
        for _ in 0..4 {
            ext.pop();
        }
    }

    // check if in cache
    if let Some(svg) = cache.get(&ext) {
        return Ok(content::Custom(http::ContentType::SVG, svg));
    }
    if ext.eq_ignore_ascii_case("folder") {
        let folder_svg = include_str!("../icons/folder.svg");
        return Ok(content::Custom(http::ContentType::SVG, folder_svg.into()));
    }

    info!("Generating icon {}", ext);
    let mut doc: svg::Document = svg::Document::new();
    doc = doc.set("viewBox", (-5, -5, 60, 60));

    use svg::node::element::path::Data;
    use svg::node::element::{Path, Rectangle, Text};

    ext.truncate(3);
    ext.make_ascii_uppercase();
    let conf = crate::config::icon_confs()
        .get(&ext)
        .map(|c| c.clone())
        .unwrap_or_else(|| {
            // generate color from sha
            use sha3::Digest;
            let mut hasher = sha3::Sha3_256::new();
            hasher.update(ext.as_bytes());
            let mut res = String::with_capacity(7);
            res.push('#');
            for e in hasher.finalize().iter().take(6) {
                res.push(std::char::from_digit((e % 16) as u32, 16).unwrap());
            }

            crate::config::IconConf {
                display_text: ext.clone(),
                color: res,
            }
        });

    // calculate brightness of icon

    let brightness: f32 = {
        let mut total = 0;
        let num = u32::from_str_radix(&conf.color[1..], 16).unwrap();
        total += num & 0xff;
        total += (num >> 8) & 0xff;
        total += (num >> 16) & 0xff;
        (total as f32) / (255.0 * 3.0)
    };

    // draw basic rect
    {
        let data = Data::new()
            .move_to((5, 50))
            .line_to((5, 15))
            .line_to((20, 0))
            .line_to((45, 0))
            .line_to((45, 50))
            .close();

        let path = Path::new()
            .set("fill", conf.color)
            .set("stroke", "none")
            .set("d", data);

        doc = doc.add(path);
    }

    // draw top edge
    {
        let data = Data::new()
            .move_to((5, 15))
            .line_to((20, 0))
            .line_to((20, 15))
            .close();

        let path = Path::new()
            .set(
                "fill",
                if brightness < 0.5 {
                    "rgba(255, 255, 255, 0.5)"
                } else {
                    "rgba(0, 0, 0, 0.3)"
                },
            )
            .set("stroke", "none")
            .set("d", data);

        doc = doc.add(path);
    }

    // add text
    {
        doc = doc.add(
            Rectangle::new()
                .set("x", "20")
                .set("y", "25")
                .set("rx", "5")
                .set("ry", "5")
                .set("height", "20")
                .set("width", "30")
                .set("fill", "#ddd"),
        );

        doc = doc.add(
            Text::new()
                .add(svg::node::Text::new(&ext))
                .set("x", "35")
                .set("y", "37")
                .set("text-anchor", "middle")
                .set("font-size", "0.9em")
                .set("font-family", "Arial, Helvetica, sans-serif")
                .set("dominant-baseline", "middle"),
        );
    }

    let mut res = Vec::with_capacity(128);
    //use std::fmt::Write;

    if let Err(e) = svg::write(&mut res, &doc) {
        warn!("Error while writing svg: {:?}", e);
        return Err(rocket::response::status::NotFound(()));
    }

    let s = String::from_utf8(res).unwrap();

    // store in cache
    cache.insert_new(ext, s.clone());

    Ok(content::Custom(http::ContentType::SVG, s))
}
