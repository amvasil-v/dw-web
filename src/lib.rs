extern crate js_sys;
extern crate web_sys;

mod utils;
mod counter_state;

use wasm_bindgen::prelude::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
