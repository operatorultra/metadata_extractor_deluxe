use std::{collections::HashMap, io::Cursor, str};

use exif::{Tag, Value};
use pdf::{file::File, object::Resolve};

use wasm_bindgen::prelude::*;

static XMP_TAG: u16 = 700;

static IPTC_TAG: u16 = 33723;

#[wasm_bindgen]
pub struct Metadata {
    // tags --> which ones are currently in place?
    // createdISO8601: publication_date
    title: String,
    author: String,
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
    subject_area: String,
    thumbnails: String,
    original_document_id: String,
}

#[wasm_bindgen]
impl Metadata {
    #[wasm_bindgen]
    pub fn get_metadata(vector: Vec<u8>, mime_type: String) -> Metadata {
        let mut title = String::new();
        let mut author = String::new();
        let mut width = String::new();
        let mut height = String::new();
        let mut resolution = HashMap::new();

        let mut copyright = String::new();
        let mut description = String::new();
        let mut make = String::new();
        let mut model = String::new();
        let mut flash_found = String::new();
        let mut gps = HashMap::new();

        let mut subject_area = HashMap::new();

        let mut thumbnails = Vec::new();

        let mut original_document_id: String = String::new();

        let mut xmp = String::new();
        let mut iptc = String::new();

        log(&format!("Getting metadata from a {}", mime_type));
        if mime_type == "application/pdf" || mime_type == "application/postscript" {
            // PDF + AI (PDF based) can contain 'PDF Info' and XMP
            let pdf_file = File::from_data(vector).unwrap();

            if let Some(ref info_dict) = pdf_file.trailer.info_dict {
                for (key, value) in info_dict {
                    if let Ok(pdf_string_value) = value.as_string() {
                        if let Ok(decoded_value) = pdf_string_value.as_str() {
                            log(&format!("FOUND PDF INFO {}: {}", key, decoded_value));
                            match key.as_str() {
                                "Title" => {
                                    title = String::from(decoded_value);
                                }

                                "Author" => {
                                    author = String::from(decoded_value);
                                }

                                "Subject" => {
                                    description = String::from(decoded_value);
                                }

                                _ => {
                                    log(&format!("Unknown PDF INFO {}: {}", key, decoded_value));
                                }
                            }
                        }
                    }
                }
            }

            if let Some(pdf_metadata_stream_ref) = pdf_file.get_root().metadata {
                if let Ok(resolved_stream) = pdf_file.get(pdf_metadata_stream_ref) {
                    if let Ok(resolved_stream_data) = resolved_stream.data() {
                        if let Ok(metadata_as_str) = str::from_utf8(resolved_stream_data) {
                            log(&format!("Found PDF Metadata: {}", metadata_as_str));
                            xmp = String::from(metadata_as_str);
                        }
                    }
                }
            }
        } else {
            let exifreader = exif::Reader::new();
            let exif = exifreader
                .read_from_container(&mut Cursor::new(vector))
                .unwrap();

            for field in exif.fields() {
                //
                if field.tag.number() == XMP_TAG {
                    if let Value::Byte(value) = &field.value {
                        let value = std::str::from_utf8(&value).unwrap();
                        xmp = value.to_string().to_string();
                    }
                }

                if field.tag.number() == IPTC_TAG {
                    if let Value::Undefined(value, _) = &field.value {
                        let value = std::str::from_utf8(&value).unwrap();
                        iptc = value.to_string().to_string();
                    }
                }

                if field.tag == Tag::SubjectArea || field.tag == Tag::SubjectLocation {
                    let value = &field.value;
                    subject_area.insert("x", value.get_uint(0).unwrap());
                    subject_area.insert("y", value.get_uint(1).unwrap());
                    if let Some(width_or_diameter) = value.get_uint(2) {
                        if let Some(height) = value.get_uint(3) {
                            subject_area.insert("width", width_or_diameter);
                            subject_area.insert("height", height);
                        } else {
                            subject_area.insert("diameter", width_or_diameter);
                        }
                    }
                }

                if field.tag == Tag::ImageDescription {
                    description = field
                        .display_value()
                        .with_unit(&exif)
                        .to_string()
                        .to_string();
                }

                if field.tag == Tag::Copyright {
                    copyright = field
                        .display_value()
                        .with_unit(&exif)
                        .to_string()
                        .to_string();
                }

                if field.tag == Tag::Make {
                    make = field
                        .display_value()
                        .with_unit(&exif)
                        .to_string()
                        .to_string();
                }

                if field.tag == Tag::Model {
                    model = field
                        .display_value()
                        .with_unit(&exif)
                        .to_string()
                        .to_string();
                }

                if field.tag == Tag::Flash {
                    flash_found = field
                        .display_value()
                        .with_unit(&exif)
                        .to_string()
                        .to_string();
                }

                if field.tag == Tag::ImageWidth || field.tag == Tag::PixelXDimension {
                    width = field
                        .display_value()
                        .with_unit(&exif)
                        .to_string()
                        .to_string();
                }

                if field.tag == Tag::ImageLength || field.tag == Tag::PixelYDimension {
                    height = field
                        .display_value()
                        .with_unit(&exif)
                        .to_string()
                        .to_string();
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
        }

        //::::::::XMP METADATA HANDLING::::::::
        if xmp.len() > 0 {
            if let Ok(root) = xmp.parse::<minidom::Element>() {
                if let Some(rdf_root) =
                    root.get_child("RDF", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
                {
                    for rdf_bag_element in rdf_root.children() {
                        //Dublin Core
                        if let Some(title_element) =
                            rdf_bag_element.get_child("title", "http://purl.org/dc/elements/1.1/")
                        {
                            title = title_element
                                .children()
                                .next()
                                .unwrap()
                                .children()
                                .next()
                                .unwrap()
                                .text();
                            log(&format!("Set title from XMP-dc-title to {}", title));
                        }

                        if let Some(rights_element) =
                            rdf_bag_element.get_child("rights", "http://purl.org/dc/elements/1.1/")
                        {
                            copyright = rights_element
                                .children()
                                .next()
                                .unwrap()
                                .children()
                                .next()
                                .unwrap()
                                .text();
                            log(&format!(
                                "Set copyright from XMP-dc-copyright to {}",
                                copyright
                            ));
                        }

                        if let Some(creator_element) =
                            rdf_bag_element.get_child("creator", "http://purl.org/dc/elements/1.1/")
                        {
                            author = creator_element
                                .children()
                                .next()
                                .unwrap()
                                .children()
                                .next()
                                .unwrap()
                                .text();
                            log(&format!("Set author from XMP-dc-creator to {}", author));
                        }

                        //XMPGImg thumbnail
                        if let Some(thumbnails_element) =
                            rdf_bag_element.get_child("Thumbnails", "http://ns.adobe.com/xap/1.0/")
                        {
                            log("Found thumbnails element");
                            for thumb_element in
                                thumbnails_element.children().next().unwrap().children()
                            {
                                thumbnails.push(HashMap::from([
                                    (
                                        "format",
                                        thumb_element
                                            .get_child(
                                                "format",
                                                "http://ns.adobe.com/xap/1.0/g/img/",
                                            )
                                            .unwrap()
                                            .text(),
                                    ),
                                    (
                                        "width",
                                        thumb_element
                                            .get_child(
                                                "width",
                                                "http://ns.adobe.com/xap/1.0/g/img/",
                                            )
                                            .unwrap()
                                            .text(),
                                    ),
                                    (
                                        "height",
                                        thumb_element
                                            .get_child(
                                                "height",
                                                "http://ns.adobe.com/xap/1.0/g/img/",
                                            )
                                            .unwrap()
                                            .text(),
                                    ),
                                    (
                                        "image",
                                        thumb_element
                                            .get_child(
                                                "image",
                                                "http://ns.adobe.com/xap/1.0/g/img/",
                                            )
                                            .unwrap()
                                            .text(),
                                    ),
                                ]));
                                log("Pushed a thumbnail");
                            }
                        }

                        //XMPMM
                        if let Some(derived_from_element) = rdf_bag_element
                            .get_child("DerivedFrom", "http://ns.adobe.com/xap/1.0/mm/")
                        {
                            if let Some(original_document_id_element) = derived_from_element
                                .get_child(
                                    "originalDocumentID",
                                    "http://ns.adobe.com/xap/1.0/sType/ResourceRef#",
                                )
                            {
                                original_document_id = original_document_id_element.text();
                            }
                        }
                    }
                }
            }
        }

        let mut resolution_vector: Vec<HashMap<String, String>> = Vec::new();
        resolution_vector.push(resolution);
        let resolution_to_json = serde_json::to_string(&resolution_vector).unwrap();

        let mut gps_vector: Vec<HashMap<String, String>> = Vec::new();
        if gps.len() > 0 {
            gps_vector.push(gps);
        }
        let gps_to_json = serde_json::to_string(&gps_vector).unwrap();
        let subject_area_to_json = serde_json::to_string(&subject_area).unwrap();
        let thumbnails_to_json = serde_json::to_string(&thumbnails).unwrap();

        return Metadata {
            title,
            author,
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
            subject_area: subject_area_to_json,
            thumbnails: thumbnails_to_json,
            original_document_id,
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
    pub fn author(&self) -> String {
        self.author.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_author(&mut self, author: String) {
        self.author = author;
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

    #[wasm_bindgen(getter)]
    pub fn subject_area(&self) -> String {
        self.subject_area.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_subject_area(&mut self, subject_area: String) {
        self.subject_area = subject_area;
    }

    #[wasm_bindgen(getter)]
    pub fn thumbnails(&self) -> String {
        self.thumbnails.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_thumbnails(&mut self, thumbnail_data: String) {
        self.thumbnails = thumbnail_data;
    }

    #[wasm_bindgen(getter)]
    pub fn original_document_id(&self) -> String {
        self.original_document_id.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_original_document_id(&mut self, document_id: String) {
        self.original_document_id = document_id;
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
