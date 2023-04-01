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
const nextButtonTargets = {
    SubmitAnswer: 0,
    NextAction: 1,
};
var nextButtonTarget = nextButtonTargets.NextAction;

const answerClickEvent = (event) => {
    if (game.check_answer(Number(event.target.dataset.num))) {
        event.target.classList.add('success');
        answer_label.textContent = 'Correct!';
    } else {
        event.target.classList.add('danger');
        answer_label.textContent = game.get_incorrent_message();
    }

    next_button.style.visibility = 'visible';
    answer_label.style.visibility = 'visible';
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
    if (!game.create_exercise()) {
        console.error('Failed to create an exercise');
        return false;
    }
    answer_input.style.visibility = 'hidden';
    const variants = game.get_answers();
    answer_label.textContent = '';
    task_label.textContent = game.get_task();
    answerButtons.forEach((btn) => {
        btn.classList.remove('success', 'danger');
        const num = Number(btn.dataset.num);
        const variant = variants[num] || '';
        btn.textContent = variant;
        btn.disabled = !variant;
        btn.style.visibility = 'visible'
    });
    next_button.style.visibility = 'hidden';
    return true;
};

const createExerciseInput = () => {
    answer_input.classList.remove('success', 'danger')
    answer_label.textContent = '';
    task_label.textContent = game.get_task();

    answerButtons.forEach((btn) => btn.style.visibility = 'hidden');
    answer_input.style.visibility = 'visible';

    next_button.textContent = "Submit";
    nextButtonTarget = nextButtonTargets.SubmitAnswer;
    next_button.style.visibility = 'visible';
    answer_input.value = "";

    return true;
};

// Init game
const createExercise = () => {
    if (!game.create_exercise()) {
        console.error('Failed to create an exercise');
        return false;
    }
    return createExerciseInput()
}

// Init Controls buttons
start_button.addEventListener('click', () => {
    try {
        answer_label.textContent = 'Loading...';
        game.fetch_words().then((res) => {
            answer_label.textContent = 'Words in vocabulary: ' + res.toString();
            if (createExercise()) {
                start_button.style.visibility = 'hidden';
            }
        });
    } catch (error) {
        console.error('Failed to fetch data');
        console.error(error);
        answer_label.textContent = error.message;
    }
});

next_button.addEventListener('click', () => {
    if (nextButtonTarget == nextButtonTargets.NextAction) {
        answer_label.style.visibility = 'hidden';
        createExercise()
    } else {
        const res = game.check_answer_input(answer_input.value);
        if (res) {
            answer_label.textContent = 'Correct!';
            answer_input.value = game.get_correct_spelling();
            answer_input.classList.add('success');
        } else {
            answer_label.textContent = game.get_incorrent_message();
            answer_input.classList.add('danger');
        }

        answer_label.style.visibility = 'visible';
        counter_state.increment_counter();
        nextButtonTarget = nextButtonTargets.NextAction;
        next_button.textContent = "Next";
    }

});
