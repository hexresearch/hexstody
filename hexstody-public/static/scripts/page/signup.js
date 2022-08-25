const uuidRegexp = /^[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12}$/i;
var usernameEl = null;
var passwordEl = null;
var passwordRepEl = null;
var inviteEl = null;
var signupTemplate = null;
const dict = {
    "en": {
        signup: "Sign up",
        username: "Username",
        password: "Password",
        reppassword: "Repeate password",
        invite: "Invite",
        alreadyTaken: "Username already taken",
        signupbtn: "SIGN UP"
    },
    "ru": {
        signup: "Регистрация",
        username: "Логин",
        password: "Пароль",
        reppassword: "Повторите пароль",
        invite: "Инвайт",
        alreadyTaken: "Имя пользователя уже занято",
        signupbtn: "ЗАРЕГИСТРИРОВАТЬСЯ"
    }
}

async function postSignUp(username, password, invite) {
    return await fetch("/signup/email",
        {
            method: "POST",
            body: JSON.stringify({ user: username, password: password, invite: {invite: invite} })
        })
};

async function loadTemplate(path) {
    const template = await (await fetch(path)).text();
    return Handlebars.compile(template);
}

function hideError(){
    document.getElementById("validation-error").parentNode.hidden = true;
}

function displayError(error){
    const validationDisplay = document.getElementById("validation-error");
    validationDisplay.textContent = error;
    validationDisplay.parentNode.hidden = false;
}

async function trySubmit() {
    hideError()
    const username = usernameEl.value;
    const password = passwordEl.value;
    const passwordRep = passwordRepEl.value;
    const invite = inviteEl.value;
    if (password == passwordRep){
        if (uuidRegexp.test(invite)) {
            const signUpResult = await postSignUp(username, password, invite);
            if (signUpResult.ok) {
                window.location.href = "/signin";
            } else {
                displayError((await signUpResult.json()).message)
            };
        } else {
            displayError("Malformed invite. UUID is expected")
        }
    } else {
        displayError("Passwords do not match!")
    }
}

async function handleLangChange(){
    const lang = document.getElementById("lang-sel").value;
    await init(lang)
    document.getElementById("lang-sel").value = lang
}

async function init(lang) {
    if(!signupTemplate){
        const [singinTemp] = await Promise.allSettled([loadTemplate("/templates/signup.html.hbs")])
        signupTemplate = singinTemp.value;
    }
    const drawUpdate = signupTemplate({lang: dict[lang]})
    document.getElementById("signup-box").innerHTML = drawUpdate
    function trySubmitOnEnter(e) {
        if (e.key === "Enter") {
            e.preventDefault();
            trySubmit();
        }
    }
    const submitButton = document.getElementById("submit");
    usernameEl = document.getElementById("username-input");
    passwordEl = document.getElementById("password-input");
    passwordRepEl = document.getElementById("password-rep-input"); 
    inviteEl = document.getElementById("invite-input");

    submitButton.onclick = trySubmit;
    usernameEl.addEventListener("keyup", trySubmitOnEnter);
    passwordEl.addEventListener("keyup", trySubmitOnEnter);
    passwordRepEl.addEventListener("keyup", trySubmitOnEnter);
    document.getElementById("lang-sel").onchange = handleLangChange;
}

document.addEventListener("DOMContentLoaded", init("en"));
