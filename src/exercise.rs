use crate::words::{Database, PartOfSpeech, Word};
use js_sys::ArrayBuffer;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::{cmp::Ordering, vec};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use wasm_bindgen::prelude::*;

const ANSWER_OPTIONS: usize = 4;

#[allow(unused)]
#[derive(EnumIter)]
pub enum ExerciseType {
    SelectDe,
    TranslateRuDe,
    SelectRu,
    GuessNounArticle,
    VerbFormRandom,
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExerciseResults {
    word: String,
    correct: usize,
    wrong: usize,
}

impl ExerciseResults {
    pub fn add(&mut self, correct: bool) {
        if correct {
            self.correct += 1;
        } else {
            self.wrong += 1;
        }
    }

    pub fn score(&self) -> i32 {
        self.correct as i32 - (self.wrong * 2) as i32
    }

    pub fn new(s: &str) -> Self {
        Self {
            correct: 0,
            wrong: 0,
            word: s.to_owned(),
        }
    }
}

impl Ord for ExerciseResults {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

impl PartialOrd for ExerciseResults {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ExerciseResults {
    fn eq(&self, other: &Self) -> bool {
        self.word == other.word
    }
}

impl Eq for ExerciseResults {}

#[derive(Debug, EnumIter)]
enum VerbFormExercise {
    PresentThird,
    Praeteritum,
    Perfect,
}

pub struct GameResults {
    results: Vec<ExerciseResults>,
    results_filename: String,
    weights: Vec<f32>,
    rand_dist: Option<WeightedIndex<f32>>,
    training: Vec<String>,
}

impl GameResults {
    pub fn new() -> Self {
        GameResults {
            results: vec![],
            results_filename: String::new(),
            weights: vec![],
            rand_dist: None,
            training: vec![],
        }
    }

    pub fn get_training_words(&self) -> &Vec<String> {
        &self.training
    }

    pub fn update_with_db(&mut self, db: &Database) {
        for word in db.words.keys() {
            let new_entry = ExerciseResults::new(word);
            if !self.results.contains(&new_entry) {
                self.results.push(new_entry);
            }
        }
        self.results.sort_unstable()
    }

    pub fn get_top_words(&self, n: usize) -> Vec<String> {
        self.results
            .iter()
            .take(n)
            .map(|r| r.word.to_owned())
            .collect()
    }

    fn select_word_to_learn(&mut self) -> &mut ExerciseResults {
        let mut rng = rand::thread_rng();
        let dist = self.rand_dist.as_ref().unwrap();
        &mut self.results[dist.sample(&mut rng)]
    }

    fn select_word_by_cmp<T>(
        &mut self,
        db: &Database,
        cmp: impl Fn(&dyn Word, &T) -> bool,
        prop: &T,
    ) -> &mut ExerciseResults {
        let mut rng = rand::thread_rng();
        let mut weights = vec![];
        let mut indices = vec![];
        for (i, &weight) in self.weights.iter().enumerate() {
            let word = &self.results[i].word;
            if let Some(w) = db.words.get(word) {
                if cmp(w.as_ref(), prop) {
                    weights.push(weight);
                    indices.push(i);
                }
            }
        }
        let dist = WeightedIndex::new(weights).unwrap();
        let idx = dist.sample(&mut rng);
        &mut self.results[indices[idx]]
    }

    fn select_word_by_pos(&mut self, db: &Database, pos: PartOfSpeech) -> &mut ExerciseResults {
        let cmp = |word: &dyn Word, prop: &PartOfSpeech| &word.get_pos() == prop;
        return self.select_word_by_cmp(db, cmp, &pos);
    }

    fn select_word_with_verb_form(
        &mut self,
        db: &Database,
        form: &VerbFormExercise,
    ) -> &mut ExerciseResults {
        let cmp = |word: &dyn Word, form: &VerbFormExercise| {
            if word.get_pos() != PartOfSpeech::Verb {
                return false;
            }
            let opt = match *form {
                VerbFormExercise::Praeteritum => word.get_verb_praeteritum(),
                VerbFormExercise::PresentThird => word.get_verb_present_third(),
                VerbFormExercise::Perfect => {
                    if word.get_verb_perfect_verb().is_none() {
                        return false;
                    }
                    word.get_verb_perfect()
                }
            };
            match opt {
                None => false,
                Some(s) if s.is_empty() => false,
                _ => true,
            }
        };
        return self.select_word_by_cmp(db, cmp, form);
    }

    pub fn update_weights(&mut self) {
        self.weights.clear();
        self.results.sort_unstable();
        let max_score = self.results.last().unwrap().score();
        let min_score = self.results.first().unwrap().score();
        self.weights.extend(
            self.results
                .iter()
                .map(|ex| (2 * max_score - min_score - ex.score() + 1) as f32),
        );
        self.rand_dist = Some(WeightedIndex::new(&self.weights).unwrap());
    }
}

pub struct Exercise {
    pub task: String,
    pub answers: Vec<String>,
    pub incorrect_message: String,
    pub correct_idx: usize,
}

fn exercise_select_de(db: &Database, word: &dyn Word) -> Exercise {
    let (options, correct_idx) = fetch_word_options(db, word);

    println!(
        "Select translation to Deutsch: {} ({})",
        word.translation(),
        word.pos_str()
    );
    let task = format!(
        "Select translation to Deutsch: {} ({})",
        word.translation(),
        word.pos_str()
    );

    let answers: Vec<String> = options.iter().map(|w| w.spelling()).collect();
    let incorrect_message = format!("Incorrect! The word is {}", word.spelling());

    Exercise {
        task,
        answers,
        incorrect_message,
        correct_idx,
    }
}

fn fetch_word_options<'a>(db: &'a Database, word: &'a dyn Word) -> (Vec<&'a dyn Word>, usize) {
    let group_id = word.get_group_id();
    let pos = word.get_pos();
    let mut rng = rand::thread_rng();
    let candidates: Vec<_> = db
        .words
        .iter()
        .filter_map(|(_, w)| {
            if w.get_group_id() == group_id && w.get_pos() == pos {
                Some(w)
            } else {
                None
            }
        })
        .collect();

    let mut options = vec![];
    let mut used_words = HashSet::new();
    options.push(word);
    used_words.insert(word.get_word());

    const MAX_ATTEMPTS: usize = 1000;
    let mut attempts = 0usize;
    while options.len() < ANSWER_OPTIONS.min(candidates.len()) {
        let cand = &***candidates.choose(&mut rng).unwrap();
        if !used_words.contains(cand.get_word()) {
            used_words.insert(cand.get_word());
            options.push(cand);
        } else {
            attempts += 1;
            if attempts >= MAX_ATTEMPTS {
                panic!("Cannot choose answer options");
            }
        }
    }

    let mut indices = (0..options.len()).collect::<Vec<usize>>();
    indices.shuffle(&mut rng);
    let mut correct = 0usize;

    let mut opt = vec![];
    for idx in indices {
        opt.push(options[idx]);
        if idx == 0 {
            correct = opt.len() - 1;
        }
    }
    (opt, correct)
}

pub fn create_exercise_with_type(
    db: &Database,
    results: &mut GameResults,
    ex_type: &ExerciseType,
) -> Option<Exercise> {
    let exercise_result = match ex_type {
        ExerciseType::VerbFormRandom => {
            let mut rng = rand::thread_rng();
            let form = VerbFormExercise::iter().choose(&mut rng).unwrap();
            results.select_word_with_verb_form(db, &form)
        }
        ExerciseType::GuessNounArticle => results.select_word_by_pos(db, PartOfSpeech::Noun),
        _ => results.select_word_to_learn(),
    };
    let word = match db.words.get(&exercise_result.word) {
        Some(w) => &**w,
        None => {
            return None;
        }
    };

    let ex = match ex_type {
        ExerciseType::SelectDe => exercise_select_de(db, word),
        _ => return None,
    };

    log::info!("{}; Answers are {:?}", ex.task, ex.answers);

    Some(ex)
}
