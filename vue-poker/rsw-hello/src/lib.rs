mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("He999llo, rsw-he333llo!");
}

#[wasm_bindgen]
pub fn hello(name: &str) -> JsValue {
    JsValue::from_str(&format!("R..dt00 shit ChangeThis 9 {}...R", name))
}

#[wasm_bindgen]
pub struct GameManager {
    //game: PostFlopGame,
}

#[wasm_bindgen]
impl GameManager {
    pub fn new() -> Self 
    {  
        Self {

        }
    }

    pub fn get_a_string(&self) -> String {
        "A!Hello from Rust!".to_string()
    }
}