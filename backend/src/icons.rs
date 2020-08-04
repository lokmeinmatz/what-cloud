use rocket::response;
use rocket::State;
use rocket::http;
use log::{info, warn};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct IconsCache(RwLock<HashMap<String, String>>);

impl IconsCache {
    pub fn empty() -> Self {
        IconsCache(RwLock::new(HashMap::new()))
    }

    pub fn get(&self, ext: &str) -> Option<String> {
        self.0.read().and_then(|hm| Ok(hm.get(ext).map(|svg| svg.clone()))).ok().flatten()
    }
}


/// Sends token on success, else error
#[get("/static/icons/<ext>")]
pub fn icons_get(mut ext: String, cache: State<IconsCache>) 
    -> Result<response::Content<String>, rocket::response::status::NotFound<()>> {

    if ext.ends_with(".svg") {
        for _ in 0..4 {
            ext.pop();
        }
    }

    // check if in cache
    if let Some(svg) = cache.get(&ext) {
        return Ok(response::Content(http::ContentType::SVG, svg));
    }
    if ext == "folder" {
        let folder_svg = include_str!("../icons/folder.svg");
        return Ok(response::Content(http::ContentType::SVG, folder_svg.into()));
    }

    info!("Generating icon {}", ext);
    let mut doc: svg::Document = svg::Document::new();
    doc = doc.set("viewBox", (0, 0, 50, 50));

    use svg::node::element::path::Data;
    use svg::node::element::Path;

    // draw basic rect 
    {
        let data = Data::new()
            .move_to(( 5, 50))
            .line_to(( 5, 10))
            .line_to((15,  0))
            .line_to((45,  0))
            .line_to((45, 50))
            .close();

        let path = Path::new()
            .set("fill", "gray")
            .set("stroke", "none")
            .set("d", data);

        doc = doc.add(path);
    }


    // draw top edge
    {
        let data = Data::new()
            .move_to(( 5, 10))
            .line_to((15,  0))
            .line_to((15, 10))
            .close();

        let path = Path::new()
            .set("fill", "#ccc")
            .set("stroke", "none")
            .set("d", data);

        doc = doc.add(path);
    }

    let mut res = Vec::with_capacity(128);
    //use std::fmt::Write;

    if let Err(e) = svg::write(&mut res, &doc) {
        warn!("Error while writing svg: {:?}", e);
        return Err(rocket::response::status::NotFound(()));
    }
    
    Ok(response::Content(http::ContentType::SVG, String::from_utf8(res).unwrap()))
}