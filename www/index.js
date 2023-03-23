import { CounterState, init_wasm_logging, WordsGame, Exercise } from 'dw-web';

const counter_state = CounterState.new();

init_wasm_logging();

const answer_names = ["button1", "button2", "button3", "button4"];
const start_button = document.getElementById("start");
const next_button = document.getElementById("next");
const answer_label = document.getElementById("answer_label");
const task_label = document.getElementById("task_label");

var game = WordsGame.create();

function set_answers_active(enabled) {
    for (const btn of answers) {
        btn.disabled = !enabled;
    }
}

function answer_listener(event) {
    set_answers_active(false);
    if (game.check_answer(event.target.answerNumber)) {
        event.target.classList.add("green");
        answer_label.textContent = "Correct!";
    } else {
        event.target.classList.add("red");
        answer_label.textContent = game.get_incorrent_message();
    }
    
    next_button.style.visibility = 'visible';    
    answer_label.style.visibility = 'visible';
    counter_state.increment_counter();
}

var answers = [];
for (let i = 0; i < 4; i++) {
    const name = answer_names[i];
    let button = document.getElementById(name);
    button.addEventListener("click", answer_listener);
    button.answerNumber = i;
    answers.push(button);
}

function create_exercise() {
    if (!game.create_exercise()) {
        console.error("Failed to create an exercise");
        return false;
    }
    let variants = game.get_answers();
    answer_label.textContent = "";
    task_label.textContent = game.get_task();
    for (let i = 0; i < 4; i++) {
        answers[i].classList.remove("green");
        answers[i].classList.remove("red");
        if (i < variants.length) {
            answers[i].textContent = variants[i];
            answers[i].disabled = false;
        } else {
            answers[i].textContent = ""
            answers[i].disabled = true;
        }
    }
    return true;
}

start_button.addEventListener("click", () => {
    try {
        answer_label.textContent = "Loading...";
        game.fetch_words().then((res) => {
            answer_label.textContent = "Words in vocabulary: " + res.toString();
            if (create_exercise()) {
                for (const btn of answers) {
                    btn.style.visibility = 'visible';
                }
                start_button.style.visibility = 'hidden';
            }
        })
    } catch (error) {
        console.error("Failed to fetch data");
        console.error(error);
        answer_label.textContent = error.message;
    }
});

next_button.addEventListener("click", () => {
    answer_label.style.visibility = 'hidden';
    if (create_exercise()) {
        next_button.visibility = 'hidden';
    }
});