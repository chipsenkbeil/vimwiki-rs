use vimwiki::{BlockElement, Language, Page, ParseError};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Output(Page<'static>);

#[wasm_bindgen]
impl Output {
    pub fn to_js(&self) -> JsValue {
        JsValue::from_serde(&self.0).unwrap()
    }

    pub fn find_all_header_regions(&self) -> Vec<JsValue> {
        self.0
            .elements
            .iter()
            .filter_map(|el| match el.as_inner() {
                BlockElement::Header(_) => {
                    Some(JsValue::from_serde(&el.region()).unwrap())
                }
                _ => None,
            })
            .collect()
    }
}

#[wasm_bindgen]
pub fn parse_vimwiki_str(s: &str) -> Result<Output, JsValue> {
    let page_res: Result<Page, ParseError> =
        Language::from_vimwiki_str(s).parse();

    match page_res {
        Ok(page) => Ok(Output(page.into_owned())),
        Err(x) => Err(x.to_string().into()),
    }
}
