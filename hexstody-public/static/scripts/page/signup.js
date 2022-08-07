const uuidRegexp = /^[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12}$/i;
var usernameEl = null;
var passwordEl = null;
var inviteEl = null;

async function postSignUp(username, password, invite) {
    return await fetch("/signup/email",
        {
            method: "POST",
            body: JSON.stringify({ user: username, password: password, invite: {invite: invite} })
        })
};

async function trySubmit() {
    const username = usernameEl.value;
    const password = passwordEl.value;
    const invite = inviteEl.value;
    const validationDisplay = document.getElementById("validation-error");
    if (uuidRegexp.test(invite)) {
        const signUpResult = await postSignUp(username, password, invite);
        if (signUpResult.ok) {
            window.location.href = "/signin";
        } else {
            validationDisplay.textContent = (await signUpResult.json()).message;
            validationDisplay.parentNode.hidden = false;
        };
    } else {
        validationDisplay.textContent = "Malformed invite. UUID is expected";
        validationDisplay.parentNode.hidden = false;
    }
}

async function init() {
    function trySubmitOnEnter(e) {
        if (e.key === "Enter") {
            e.preventDefault();
            trySubmit();
        }
    }
    const submitButton = document.getElementById("submit");
    usernameEl = document.getElementById("username-input");
    passwordEl = document.getElementById("password-input");
    inviteEl = document.getElementById("invite-input");

    submitButton.onclick = trySubmit;
    usernameEl.addEventListener("keyup", trySubmitOnEnter);
    passwordEl.addEventListener("keyup", trySubmitOnEnter);
}

document.addEventListener("DOMContentLoaded", init);
