import { CounterState } from 'dw-web';

const counter_state = CounterState.new();

const answer_names = ["button1", "button2", "button3", "button4"];
const start_button = document.getElementById("start");
const next_button = document.getElementById("next");
const answer_label = document.getElementById("answer_label");

function answer_listener(event) {
	event.target.classList.add("green");
	next_button.style.visibility = 'visible';
	for (const btn of answers) {
	  btn.disabled = true;
    }
	answer_label.textContent = "Correct!";
	answer_label.style.visibility = 'visible';
    counter_state.increment_counter();
}

var answers = [];
for (const name of answer_names) {
	let button = document.getElementById(name);
	button.addEventListener("click", answer_listener);
	answers.push(button);
}

start_button.addEventListener("click", () => {
  start_button.style.visibility = 'hidden';
  answer_label.style.visibility = 'hidden';
  for (const btn of answers) {
	  btn.style.visibility = 'visible';
  }
});

next_button.addEventListener("click", () => {
  for (const btn of answers) {
	  btn.classList.remove("green");
	  btn.disabled = false;
	  next_button.visibility = 'hidden';
  }
  answer_label.style.visibility = 'hidden';
});