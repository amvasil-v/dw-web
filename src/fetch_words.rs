use js_sys::ArrayBuffer;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[wasm_bindgen]
pub async fn fetch_words() -> Result<usize, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let url = "https://api.github.com/repos/amvasil-v/das_woerterbuch/contents/woerterbuch.xlsx";
    let request = Request::new_with_str_and_init(url, &opts)?;

    request
        .headers()
        .set("Accept", "application/vnd.github.raw")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    if !resp.ok() {
        return Ok(0)
    }

    let array: ArrayBuffer = resp.array_buffer()?.dyn_into()?;
    
    Ok(array.byte_length() as usize)
}
