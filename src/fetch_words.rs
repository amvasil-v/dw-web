use calamine::Reader;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn fetch_words() -> Result<usize, JsError> {
    let url = "https://api.github.com/repos/amvasil-v/das_woerterbuch/contents/woerterbuch.xlsx";
    let client = reqwest::Client::new();
    let request = client.get(url).header(
        reqwest::header::ACCEPT, "application/vnd.github.v3.raw");

    let body = request.send().await?.bytes().await?;
    
    let cursor = std::io::Cursor::new(body);
    let mut workbook = calamine::open_workbook_auto_from_rs(cursor)?;

    let range = match workbook.worksheet_range("Words") {
        None => return Err(JsError::new("No sheet called Words")),
        Some(r) => r?
    };

    Ok(range.rows().count())
}
