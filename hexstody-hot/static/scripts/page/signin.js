async function postSignIn(email, password) {
    return await fetch("/signin/email",
        {
            method: "POST",
            body: JSON.stringify({ user: email, password: password })
        })
};

async function trySubmit() {
    const email = document.getElementById("signinEmail").value;
    const password = document.getElementById("signinPassword").value;
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
    const submitButton = document.getElementById("submit");
    submitButton.onclick = trySubmit;
}

document.addEventListener("DOMContentLoaded", init);