var emailEl = null;
var passwordEl = null;

async function postSignUp(email, password) {
    return await fetch("/signup/email",
        {
            method: "POST",
            body: JSON.stringify({ user: email, password: password })
        })
};

async function trySubmit() {
    const email = emailEl.value;
    const password = passwordEl.value;
    const signUpResult = await postSignUp(email, password);
    if (signUpResult.ok) {
      window.location.href = "/signin";

    } else {
        const validationDisplay = document.getElementById("validationError");
        validationDisplay.textContent = (await signUpResult.json()).message;
        validationDisplay.hidden = false;
    };
}

async function init() {
    function trySubmitOnEnter(e){
        if (e.key === "Enter") {
            e.preventDefault();
            trySubmit();
        }
    }
    const submitButton = document.getElementById("submit");
    emailEl = document.getElementById("signUpEmail");
    passwordEl = document.getElementById("signUpPassword");

    submitButton.onclick = trySubmit;
    emailEl.addEventListener("keyup",trySubmitOnEnter);
    passwordEl.addEventListener("keyup", trySubmitOnEnter);
}

document.addEventListener("DOMContentLoaded", init);
