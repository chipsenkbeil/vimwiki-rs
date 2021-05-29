use wasm_bindgen::{convert::FromWasmAbi, prelude::*};

// From https://github.com/rustwasm/wasm-bindgen/issues/2231#issuecomment-656293288
pub fn cast_value<T: FromWasmAbi<Abi = u32>>(
    js: JsValue,
    classname: &str,
) -> Result<T, JsValue> {
    use js_sys::{Object, Reflect};
    let ctor_name = Object::get_prototype_of(&js).constructor().name();
    if ctor_name == classname {
        let ptr = Reflect::get(&js, &JsValue::from_str("ptr"))?;
        let ptr_u32: u32 = ptr.as_f64().ok_or(JsValue::NULL)? as u32;
        let value = unsafe { T::from_abi(ptr_u32) };
        Ok(value)
    } else {
        Err(JsValue::NULL)
    }
}
