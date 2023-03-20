use wasm_bindgen::prelude::*;


#[wasm_bindgen]
pub async fn fetch_words() -> Result<usize, JsValue> {
    let url = "https://api.github.com/repos/amvasil-v/das_woerterbuch/contents/woerterbuch.xlsx";
    let body = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    
    Ok(body.len())
}
