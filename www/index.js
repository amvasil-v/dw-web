import { CounterState, init_wasm_logging, WordsGame, Exercise } from 'dw-web';

const counter_state = CounterState.new();

init_wasm_logging();

const game = WordsGame.create();

const start_button = document.getElementById('start');
const next_button = document.getElementById('next');
const answer_label = document.getElementById('answer_label');
const task_label = document.getElementById('task_label');

// Init buttons
const answerButtons = document.querySelectorAll('.btn-answer');

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

// Init game
const createExercise = () => {
    if (!game.create_exercise()) {
        console.error('Failed to create an exercise');
        return false;
    }
    const variants = game.get_answers();
    answer_label.textContent = '';
    task_label.textContent = game.get_task();
    answerButtons.forEach((btn) => {
        btn.classList.remove('success', 'danger');
        const num = Number(btn.dataset.num);
        const variant = variants[num] || '';
        btn.textContent = variant;
        btn.disabled = !variant;
    });

    return true;
};

// Init Controls buttons
start_button.addEventListener('click', () => {
    try {
        answer_label.textContent = 'Loading...';
        game.fetch_words().then((res) => {
            answer_label.textContent = 'Words in vocabulary: ' + res.toString();
            if (createExercise()) {
                for (const btn of answers) {
                    btn.style.visibility = 'visible';
                }
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
    answer_label.style.visibility = 'hidden';
    if (create_exercise()) {
        next_button.visibility = 'hidden';
    }
});
