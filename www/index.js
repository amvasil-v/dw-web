import { CounterState, init_wasm_logging, WordsGame, Exercise } from 'dw-web';

const counter_state = CounterState.new();

init_wasm_logging();

const game = WordsGame.create();

const start_button = document.getElementById('start');
const next_button = document.getElementById('next');
const answer_label = document.getElementById('answer_label');
const task_label = document.getElementById('task_label');
const answer_input = document.getElementById('answer_input');

// Init buttons
const answerButtons = document.querySelectorAll('.btn-answer');
const answerButtonsContainer = document.querySelector('.choices-wrapper');
const defaultButtonsHeight = answerButtonsContainer.style.height;
const defaultInputHeight = answer_input.style.height;

answer_input.addEventListener('change', () => {
    start_button.click();
})

const answerClickEvent = (event) => {
    if (game.check_answer(Number(event.target.dataset.num))) {
        event.target.classList.add('success');
        answer_label.textContent = game.get_correct_message();
    } else {
        event.target.classList.add('danger');
        answer_label.textContent = game.get_incorrent_message();
    }

    next_button.style.visibility = 'visible';
    counter_state.increment_counter();
};

answerButtons.forEach((btn) => btn.addEventListener('click', answerClickEvent));

const prepareGame = () => {
    answerButtons.forEach((btn) => btn.style.visibility = 'hidden');
    next_button.style.visibility = 'hidden';
    answer_input.style.visibility = 'hidden';
}

prepareGame();

const createExerciseChoise = () => {
    answer_input.style.visibility = 'hidden';
    answer_input.style.height = "0px";
    const variants = game.get_answers();
    answer_label.textContent = "Select one answer";
    task_label.textContent = game.get_task();
    answerButtonsContainer.style.height = defaultButtonsHeight;
    answerButtons.forEach((btn) => {
        btn.classList.remove('success', 'danger');
        const num = Number(btn.dataset.num);
        const variant = variants[num] || '';
        btn.textContent = variant;
        btn.disabled = !variant;
        btn.style.visibility = 'visible'
    });
    start_button.style.visibility = 'hidden';
    return true;
};

const createExerciseInput = () => {
    answer_input.classList.remove('success', 'danger')
    answer_label.textContent = "Type in the answer. ß=ss, ö=oe etc.";
    task_label.textContent = game.get_task();

    answerButtons.forEach((btn) => btn.style.visibility = 'hidden');
    answer_input.style.height = defaultInputHeight;
    answer_input.style.visibility = 'visible';
    answerButtonsContainer.style.height = "0px";
    
    answer_input.value = "";
    answer_input.focus();
    start_button.style.visibility = 'visible';

    return true;
};

// Init game
const createExercise = () => {
    if (!game.create_exercise()) {
        console.error('Failed to create an exercise');
        return false;
    }
    next_button.style.visibility = 'hidden';
    if (game.is_exercise_input()) {
        return createExerciseInput();
    } else {
        return createExerciseChoise();
    }

}

const onSubmit = () => {
    start_button.style.visibility = 'hidden';
    const res = game.check_answer_input(answer_input.value);
    if (res) {
        answer_label.textContent = game.get_correct_message();
        answer_input.value = game.get_correct_spelling();
        answer_input.classList.add('success');
    } else {
        answer_label.textContent = game.get_incorrent_message();
        answer_input.classList.add('danger');
    }

    counter_state.increment_counter();
    next_button.style.visibility = 'visible';
    next_button.focus();
}

const onStart = () => {
    try {
        answer_label.textContent = 'Loading...';
        game.fetch_words().then((res) => {
            answer_label.textContent = 'Words in vocabulary: ' + res.toString();
            setupSubmitButton();
            createExercise();
        });
    } catch (error) {
        console.error('Failed to fetch data');
        console.error(error);
        answer_label.textContent = error.message;
    }
}

const setupSubmitButton = () => {
    start_button.textContent = "Submit";
    start_button.classList.remove('success');
    start_button.classList.add('warning');
    start_button.style.visibility = 'hidden';
    start_button.removeEventListener('click', onStart);
    start_button.addEventListener('click', onSubmit);
}

// Init Controls buttons
start_button.addEventListener('click', onStart);
next_button.addEventListener('click', () => {
    createExercise()
});
