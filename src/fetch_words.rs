use base64::Engine;
use calamine::Reader;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init() {
    wasm_logger::init(wasm_logger::Config::default());
}

#[wasm_bindgen]
pub async fn fetch_words() -> Result<usize, JsError> {
    let url = "https://api.github.com/repos/amvasil-v/das_woerterbuch/contents/woerterbuch.xlsx";
    let media = "application/vnd.github.v3.raw";
    let client = reqwest::Client::new();
    let request = client.get(url).header(reqwest::header::ACCEPT, media);

    log::info!("Sending request");

    let response = request.send().await?;

    let json_response = match response.headers().get("Content-Type") {
        None => {
            log::warn!("No Content-Type header");
            false
        }
        Some(accept) => accept.to_str()?.contains("application/json"),
    };

    let range_res = if !json_response {
        log::info!("Parsing raw file");
        let body = response.bytes().await?;
        let cursor = std::io::Cursor::new(body);
        let mut workbook = calamine::open_workbook_auto_from_rs(cursor)?;
        workbook.worksheet_range("Words")
    } else {
        log::info!("Parsing json response with file");
        let json = response.text().await?;
        let parsed = json::parse(&json)?;
        let content = parsed["content"].as_str().unwrap();
        let mut binary: Vec<u8> = vec![];
        let ext = content
            .split('\n')
            .map(|s| base64::engine::general_purpose::STANDARD.decode(s).unwrap());
        for mut v in ext.into_iter() {
            binary.append(&mut v);
        }
        let cursor = std::io::Cursor::new(binary);
        let mut workbook = calamine::open_workbook_auto_from_rs(cursor)?;
        workbook.worksheet_range("Words")
    };

    let range = match range_res {
        None => return Err(JsError::new("No sheet called Words")),
        Some(r) => r?,
    };

    let rows_count = range.rows().count();
    log::info!("Parsec xlsx with {} words", rows_count);
    Ok(rows_count)
}
