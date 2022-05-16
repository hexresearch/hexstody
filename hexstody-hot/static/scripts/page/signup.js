async function postSignUp(email, password) {
    return await fetch("/signup/email",
        {
            method: "POST",
            body: JSON.stringify({ user: email, password: password })
        })
};

async function trySubmit() {
    const email = document.getElementById("signupEmail").value;
    const password = document.getElementById("signupPassword").value;
    const signUpResult = await postSignUp(email, password);
    if (signUpResult.ok) {
        window.location.href = "/overview";

    } else {
        const validationDisplay = document.getElementById("validationError");
        validationDisplay.textContent = (await signUpResult.json()).message;
        validationDisplay.hidden = false;
    };
}

async function init() {
    const submitButton = document.getElementById("submit");
    submitButton.onclick = trySubmit;
}

document.addEventListener("DOMContentLoaded", init);