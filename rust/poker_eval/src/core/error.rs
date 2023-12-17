use std::fmt::Display;

use wasm_bindgen::JsValue;



#[derive(Debug)]
pub struct PokerError {
    details: String,
}

impl PokerError {
    pub fn from_str(msg: &str) -> PokerError {
        PokerError {
            details: msg.to_string(),
        }
    }
    pub fn from_string(msg: String) -> PokerError {
        PokerError { details: msg }
    }
}

impl Display for PokerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

//https://medium.com/@gj.denhertog/enhancing-error-handling-in-wasm-bindgen-with-custom-types-for-rust-ee6fd25cd31e
impl From<PokerError> for JsValue {
    fn from(failure: PokerError) -> Self {
        js_sys::Error::new(&failure.to_string()).into()
    }
}
