use base64::Engine;
use calamine::Reader;
use core::panic;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::exercise::*;
use crate::words::*;

#[wasm_bindgen]
pub fn init_wasm_logging() {
    wasm_logger::init(wasm_logger::Config::default());
}

#[wasm_bindgen]
pub struct WordsGame {
    db: Database,
    results: GameResults,
    exercise: Option<Exercise>,
}

#[wasm_bindgen]
impl WordsGame {
    pub fn create() -> WordsGame {
        WordsGame {
            db: Database::new(),
            results: GameResults::new(),
            exercise: None,
        }
    }

    pub async fn fetch_words(&mut self) -> Result<usize, JsError> {
        let url =
            "https://api.github.com/repos/amvasil-v/das_woerterbuch/contents/woerterbuch.xlsx";
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
        log::info!("Parsed xlsx with {} words", rows_count);

        for row in range.rows().skip(2) {
            let mut map: HashMap<usize, String> =
                row.iter().map(|dt| dt.to_string()).enumerate().collect();
            let pos = get_part_of_speech(&map);
            let word = match pos {
                "n" => Box::new(Noun::new(&mut map, &mut self.db)) as Box<dyn Word>,
                "v" => Box::new(Verb::new(&mut map, &mut self.db)),
                "adj" => Box::new(Adjective::new(&mut map, &mut self.db)),
                "adv" => Box::new(Adverb::new(&mut map, &mut self.db)),
                "prep" => Box::new(Preposition::new(&mut map, &mut self.db)),
                _ => continue,
            };
            self.db.words.insert(word.get_word().to_owned(), word);
        }

        self.results.update_with_db(&self.db);
        self.results.update_weights();

        Ok(self.db.words.keys().count())
    }

    pub fn create_exercise(&mut self) -> bool {
        self.exercise =
            create_exercise_with_type(&self.db, &mut self.results, &ExerciseType::TranslateRuDe);
        self.exercise.is_some()
    }

    pub fn get_answers(&self) -> JsValue {
        let ex = match &self.exercise {
            None => return JsValue::UNDEFINED,
            Some(ex) => ex,
        };
        return JsValue::from(
            ex.answers
                .iter()
                .map(|x| JsValue::from_str(x))
                .collect::<js_sys::Array>(),
        );
    }

    pub fn get_task(&self) -> JsValue {
        match &self.exercise {
            None => JsValue::UNDEFINED,
            Some(ex) => JsValue::from_str(&ex.task),
        }
    }

    pub fn check_answer(&self, answer: usize) -> bool {
        match &self.exercise {
            None => false,
            Some(ex) => ex.correct_idx == answer,
        }
    }

    pub fn get_incorrent_message(&self) -> JsValue {
        match &self.exercise {
            None => JsValue::UNDEFINED,
            Some(ex) => JsValue::from_str(&ex.incorrect_message),
        }
    }

    pub fn is_exercise_input(&self) -> bool {
        match &self.exercise {
            None => false,
            Some(ex) => match ex.ex_type {
                ExerciseType::TranslateRuDe | ExerciseType::VerbFormRandom => true,
                _ => false,
            },
        }
    }

    pub fn check_answer_input(&self, answer: &str) -> bool {
        if let Some(ex) = &self.exercise {
            match ex.ex_type {
                ExerciseType::TranslateRuDe | ExerciseType::VerbFormRandom => {
                    ex.check_input_spelling(answer)
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn get_correct_spelling(&self) -> JsValue {
        match &self.exercise {
            None => JsValue::UNDEFINED,
            Some(ex) => JsValue::from_str(&ex.correct_spelling),
        }
    }
}
