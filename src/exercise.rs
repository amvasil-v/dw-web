use crate::words::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::{cmp::Ordering, vec};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const ANSWER_OPTIONS: usize = 4;

#[allow(unused)]
#[derive(EnumIter, Debug)]
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

pub struct ExerciseDataBullets {
    pub answers: Vec<String>,
    pub correct_idx: usize,
}

pub struct ExerciseDataArticle {
    pub data: ExerciseDataBullets,
    pub correct_message: String,
}

pub struct ExerciseDataInput {
    pub correct_spelling: String,
}

pub struct ExerciseDataVerbForm {
    form: VerbFormExercise,
    data: ExerciseDataInput,
}

pub enum ExerciseData {
    Bullets(ExerciseDataBullets),
    TextInput(ExerciseDataInput),
    VerbForm(ExerciseDataVerbForm),
    Article(ExerciseDataArticle),
}

pub struct Exercise {
    pub ex_type: ExerciseType,
    pub task: String,
    pub incorrect_message: String,
    pub data: ExerciseData,
}

impl Exercise {
    fn check_input_spelling(&self, input: &str) -> bool {
        if let ExerciseData::TextInput(data) = &self.data {
            check_spelling_simple(input, &data.correct_spelling)
        } else {
            false
        }
    }

    fn check_verb_form_spelling(&self, input: &str) -> bool {
        if let ExerciseData::VerbForm(data) = &self.data {
            let correct = &data.data.correct_spelling;
            match data.form {
                VerbFormExercise::Perfect => check_spelling_perfect(input, correct),
                _ => check_spelling_simple(input, correct),
            }
        } else {
            false
        }
    }

    pub fn check_spelling(&self, input: &str) -> bool {
        match self.data {
            ExerciseData::TextInput(_) => self.check_input_spelling(input),
            ExerciseData::VerbForm(_) => self.check_verb_form_spelling(input),
            _ => false,
        }
    }

    pub fn get_correct_spelling(&self) -> &str {
        match &self.data {
            ExerciseData::TextInput(data) => &data.correct_spelling,
            ExerciseData::VerbForm(data) => &data.data.correct_spelling,
            _ => "",
        }
    }

    pub fn get_correct_message(&self) -> &str {
        match &self.data {
            ExerciseData::Article(data) => &data.correct_message,
            _ => "Correct!",
        }
    }

    pub fn check_answer(&self, answer: usize) -> bool {
        match &self.data {
            ExerciseData::Article(data) => data.data.correct_idx == answer,
            ExerciseData::Bullets(data) => data.correct_idx == answer,
            _ => false,
        }
    }

    pub fn get_answers(&self) -> Option<&Vec<String>> {
        match &self.data {
            ExerciseData::Article(data) => Some(&data.data.answers),
            ExerciseData::Bullets(data) => Some(&data.answers),
            _ => None,
        }
    }
}

fn exercise_select_de(db: &Database, word: &dyn Word) -> Exercise {
    let (options, correct_idx) = fetch_word_options(db, word);

    let task = format!(
        "Select translation to Deutsch: {} ({})",
        word.translation(),
        word.pos_str()
    );

    let answers: Vec<String> = options.iter().map(|w| w.spelling()).collect();
    let incorrect_message = format!("Incorrect! The word is {}", word.spelling());
    let data = ExerciseDataBullets {
        answers,
        correct_idx,
    };

    Exercise {
        ex_type: ExerciseType::SelectDe,
        task,
        incorrect_message,
        data: ExerciseData::Bullets(data),
    }
}

fn exercise_select_ru(db: &Database, word: &dyn Word) -> Exercise {
    let (options, correct_idx) = fetch_word_options(db, word);

    println!(
        "Select translation to Russian: {} ({})",
        word.spelling(),
        word.pos_str()
    );

    let task = format!(
        "Select translation to Russian: {} ({})",
        word.spelling(),
        word.pos_str()
    );

    let answers: Vec<String> = options.iter().map(|w| w.translation().to_owned()).collect();
    let incorrect_message = format!("Incorrect! The tranlation is {}", word.translation());
    let data = ExerciseDataBullets {
        answers,
        correct_idx,
    };

    Exercise {
        ex_type: ExerciseType::SelectRu,
        task,
        incorrect_message,
        data: ExerciseData::Bullets(data),
    }
}

fn exercise_translate_to_de(word: &dyn Word) -> Exercise {
    let mut task = format!(
        "Translate to German: {} ({})",
        word.translation(),
        word.pos_str()
    );
    let help = word.get_help();
    if !help.is_empty() {
        task.push_str(&format!(". Hint: {}", help));
    }
    let correct_spelling = word.spelling();
    let incorrect_message = format!("Incorrect! The word is {}", word.spelling());

    Exercise {
        ex_type: ExerciseType::TranslateRuDe,
        task,
        incorrect_message,
        data: ExerciseData::TextInput(ExerciseDataInput { correct_spelling }),
    }
}

fn exercise_guess_noun_article(word: &dyn Word) -> Exercise {
    let task = format!(
        "Select the correct article for the noun: {}",
        capitalize_noun(word.get_word())
    );

    let answers: Vec<String> = NounArticle::iter().map(|a| a.answer_bullet_str()).collect();
    let incorrect_message = format!(
        "Incorrect! The article is {} - {}",
        word.spelling(),
        word.translation()
    );

    let correct_idx = answers
        .iter()
        .enumerate()
        .find_map(|(i, x)| {
            if x == &word.get_article().unwrap().answer_bullet_str() {
                Some(i)
            } else {
                None
            }
        })
        .unwrap();
    let data = ExerciseDataArticle {
        data: ExerciseDataBullets {
            answers,
            correct_idx,
        },
        correct_message: format!("Correct! {} - {}", word.spelling(), word.translation()),
    };

    Exercise {
        ex_type: ExerciseType::GuessNounArticle,
        task,
        incorrect_message,
        data: ExerciseData::Article(data),
    }
}

fn exercise_verb_form(word: &dyn Word, form: VerbFormExercise) -> Exercise {
    let task = format!(
        "{} [ {} - {} ]",
        match form {
            VerbFormExercise::PresentThird => "Add verb in present tense: Er ... jetzt",
            VerbFormExercise::Praeteritum => "Add verb in Präteritum : Er ... einst",
            VerbFormExercise::Perfect => "Add verb in Perfekt : Er ... ... gestern",
        },
        word.get_word(),
        word.translation()
    );

    let correct_spelling = match form {
        VerbFormExercise::PresentThird => word.get_verb_present_third().unwrap().to_owned(),
        VerbFormExercise::Praeteritum => word.get_verb_praeteritum().unwrap().to_owned(),
        VerbFormExercise::Perfect => word.get_verb_perfect_full().unwrap(),
    };
    let incorrect_message = format!("Incorrect! The form is {}", correct_spelling);
    let data = ExerciseDataVerbForm {
        data: ExerciseDataInput { correct_spelling },
        form,
    };

    Exercise {
        ex_type: ExerciseType::VerbFormRandom,
        task,
        incorrect_message,
        data: ExerciseData::VerbForm(data),
    }
}

pub fn exercise_verb_form_random(word: &dyn Word) -> Exercise {
    let mut rng = rand::thread_rng();
    let form = VerbFormExercise::iter().choose(&mut rng).unwrap();
    exercise_verb_form(word, form)
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
        ExerciseType::TranslateRuDe => exercise_translate_to_de(word),
        ExerciseType::SelectRu => exercise_select_ru(db, word),
        ExerciseType::GuessNounArticle => exercise_guess_noun_article(word),
        ExerciseType::VerbFormRandom => exercise_verb_form_random(word),
        _ => return None,
    };

    Some(ex)
}
