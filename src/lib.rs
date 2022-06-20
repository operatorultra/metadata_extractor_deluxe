use exif::{Tag, Value};

use std::{collections::HashMap, io::Cursor};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Metadata {
    // tags --> which ones are currently in place?
    // createdISO8601: publication_date
    title: String,
    width: String,
    height: String,
    resolution: String, // dpi (if dots per inch is 0 set it to 72)
    make: String,
    model: String,
    flash_found: String,
    copyright: String,
    description: String,
    gps: String,
    xmp: String,
    iptc: String,
}

#[wasm_bindgen]
impl Metadata {
    #[wasm_bindgen]
    pub fn get_metadata(vector: Vec<u8>) -> Metadata {
        let xmp_tag = 700;
        let iptc_tag = 33723;

        let mut file = Cursor::new(vector);

        let exifreader = exif::Reader::new();
        let exif = exifreader.read_from_container(&mut file).unwrap();

        let mut title = String::new();
        let mut width = String::new();
        let mut height = String::new();
        let mut resolution = HashMap::new();

        let mut copyright = String::new();
        let mut description = String::new();
        let mut make = String::new();
        let mut model = String::new();
        let mut flash_found = String::new();
        let mut gps = HashMap::new();

        let mut xmp = String::new();
        let mut iptc = String::new();

        for field in exif.fields() {
            //
            if field.tag.number() == xmp_tag {
                if let Value::Byte(value) = &field.value {
                    let value = std::str::from_utf8(&value).unwrap();
                    xmp.push_str(value.to_string().as_str());
                }
            }

            if field.tag.number() == iptc_tag {
                if let Value::Undefined(value, _) = &field.value {
                    let value = std::str::from_utf8(&value).unwrap();
                    iptc.push_str(value.to_string().as_str());
                }
            }

            if field.tag == Tag::ImageDescription {
                description.push_str(field.display_value().with_unit(&exif).to_string().as_str());
            }

            if field.tag == Tag::Copyright {
                copyright.push_str(field.display_value().with_unit(&exif).to_string().as_str());
            }

            if field.tag == Tag::Make {
                make.push_str(field.display_value().with_unit(&exif).to_string().as_str());
            }

            if field.tag == Tag::Model {
                model.push_str(field.display_value().with_unit(&exif).to_string().as_str());
            }

            if field.tag == Tag::Flash {
                flash_found.push_str(field.display_value().with_unit(&exif).to_string().as_str());
            }

            if field.tag == Tag::ImageWidth {
                width.push_str(field.display_value().with_unit(&exif).to_string().as_str());
            }

            if field.tag == Tag::ImageLength {
                height.push_str(field.display_value().with_unit(&exif).to_string().as_str());
            }

            if field.tag == Tag::XResolution {
                resolution.insert(
                    "x".to_string(),
                    field.display_value().with_unit(&exif).to_string(),
                );
            }
            if field.tag == Tag::YResolution {
                resolution.insert(
                    "y".to_string(),
                    field.display_value().with_unit(&exif).to_string(),
                );
            }

            if field.tag == Tag::GPSLatitude {
                gps.insert(
                    "latitude".to_string(),
                    field.display_value().with_unit(&exif).to_string(),
                );
            }
            if field.tag == Tag::GPSLongitude {
                gps.insert(
                    "longitude".to_string(),
                    field.display_value().with_unit(&exif).to_string(),
                );
            }

            // log(&format!(
            //     "{:?} {:?} {}",
            //     field.tag,
            //     field.ifd_num,
            //     field.display_value().with_unit(&exif),
            // ));
        }

        //::::::::XMP METADATA HANDLING::::::::
        if xmp.len() > 0 {
            let root: minidom::Element = xmp.parse().unwrap();

            // Children and Grand children are ways of digging into the XML
            // to get to the Title of the document
            let children = root
                .children()
                .next()
                .unwrap()
                .children()
                .collect::<Vec<_>>();
            let grand_children = children
                .iter()
                .next()
                .unwrap()
                .children()
                .collect::<Vec<_>>();

            grand_children.iter().for_each(|child| {
                if child.name() == "title" {
                    let document_title = child
                        .children()
                        .next()
                        .unwrap()
                        .get_child("li", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
                        .unwrap()
                        .text();

                    title.push_str(document_title.to_string().as_str());

                    // log(&format!("{:#?} ,{:#?}", child, document_title));
                }
            });
        }

        let mut resolution_vector: Vec<HashMap<String, String>> = Vec::new();
        resolution_vector.push(resolution);
        let resolution_to_json = serde_json::to_string(&resolution_vector).unwrap();

        let mut gps_vector: Vec<HashMap<String, String>> = Vec::new();
        if gps.len() > 0 {
            gps_vector.push(gps);
        }
        let gps_to_json = serde_json::to_string(&gps_vector).unwrap();

        return Metadata {
            title,
            width,
            height,
            resolution: resolution_to_json,
            make,
            model,
            flash_found,
            copyright,
            description,
            gps: gps_to_json,
            xmp,
            iptc,
        };
    }

    // The following getters & setters are necessary for the wasm_bindgen to work
    // and to have access to the Metadata struct in the frontend (as JS object)

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> String {
        self.width.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_width(&mut self, width: String) {
        self.width = width;
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> String {
        self.height.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_height(&mut self, height: String) {
        self.height = height;
    }

    #[wasm_bindgen(getter)]
    pub fn resolution(&self) -> String {
        self.resolution.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_resolution(&mut self, resolution: String) {
        self.resolution = resolution;
    }

    #[wasm_bindgen(getter)]
    pub fn make(&self) -> String {
        self.make.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_make(&mut self, make: String) {
        self.make = make;
    }

    #[wasm_bindgen(getter)]
    pub fn model(&self) -> String {
        self.model.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }

    #[wasm_bindgen(getter)]
    pub fn copyright(&self) -> String {
        self.copyright.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_copyright(&mut self, copyright: String) {
        self.copyright = copyright;
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> String {
        self.description.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    #[wasm_bindgen(getter)]
    pub fn flash_found(&self) -> String {
        self.flash_found.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_flash_found(&mut self, flash_found: String) {
        self.flash_found = flash_found;
    }

    #[wasm_bindgen(getter)]
    pub fn gps(&self) -> String {
        self.gps.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_gps(&mut self, gps: String) {
        self.gps = gps;
    }

    #[wasm_bindgen(getter)]
    pub fn xmp(&self) -> String {
        self.xmp.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_xmp(&mut self, xmp: String) {
        self.xmp = xmp;
    }

    #[wasm_bindgen(getter)]
    pub fn iptc(&self) -> String {
        self.iptc.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_iptc(&mut self, iptc: String) {
        self.iptc = iptc;
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Logging helper JS  function
///////////////////////////////////////////////////////////////////////////////

// Javascript Debug tools
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

}
