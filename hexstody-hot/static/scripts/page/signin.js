var emailEl = null;
var passwordEl = null;

async function postSignIn(email, password) {
    return await fetch("/signin/email",
        {
            method: "POST",
            body: JSON.stringify({ user: email, password: password })
        })
};

async function trySubmit() {
    const email = emailEl.value;
    const password = passwordEl.value;
    const signInResult = await postSignIn(email, password);
    if (signInResult.ok) {
        window.location.href = "/overview";

    } else {
        const validationDisplay = document.getElementById("validationError");
        validationDisplay.textContent = (await signInResult.json()).message;
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
    
    emailEl = document.getElementById("signInEmail");
    passwordEl = document.getElementById("signInPassword");

    submitButton.onclick = trySubmit;
    emailEl.addEventListener("keyup",trySubmitOnEnter);
    passwordEl.addEventListener("keyup", trySubmitOnEnter);
};

document.addEventListener("DOMContentLoaded", init);