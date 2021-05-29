use vimwiki::{self as v, Language, ParseError};
use wasm_bindgen::prelude::*;

mod elements;
pub use elements::*;

mod utils;

#[wasm_bindgen]
pub fn parse_vimwiki_str(s: &str) -> Result<Page, JsValue> {
    let page_res: Result<v::Page, ParseError> =
        Language::from_vimwiki_str(s).parse();

    match page_res {
        Ok(page) => Ok(Page::from(page.into_owned())),
        Err(x) => Err(x.to_string().into()),
    }
}
