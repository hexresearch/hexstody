import { getBrowserLanguage } from "../localize.js";
import { loadTemplate, initDropDowns } from "../common.js";

const uuidRegexp = /^[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12}$/i;

var usernameEl = null;
var passwordEl = null;
var passwordRepEl = null;
var inviteEl = null;

var headerTemplate = null;
var signupTemplate = null;

var headerTranslations = null;
var signUpPageTranslations = null;

async function postSignUp(username, password, invite) {
    return await fetch("/signup/email",
        {
            method: "POST",
            body: JSON.stringify({ user: username, password: password, invite: { invite: invite } })
        })
};

function hideError() {
    document.getElementById("validation-error").hidden = true;
}

function displayError(error) {
    const validationDisplay = document.getElementById("validation-error");
    validationDisplay.getElementsByTagName("span")[0].textContent = error;
    validationDisplay.hidden = false;
}

async function trySubmit() {
    hideError()
    const username = usernameEl.value;
    const password = passwordEl.value;
    const passwordRep = passwordRepEl.value;
    const invite = inviteEl.value;
    if (password == passwordRep) {
        if (uuidRegexp.test(invite)) {
            const signUpResult = await postSignUp(username, password, invite);
            if (signUpResult.ok) {
                window.location.href = "/signin";
            } else {
                displayError((await signUpResult.json()).message);
            };
        } else {
            displayError(signUpPageTranslations.inviteError);
        }
    } else {
        displayError(signUpPageTranslations.passwordError);
    }
}

async function handleLangChange(lang) {
    await init(lang);
}

async function init(lang) {
    if (!signupTemplate) {
        const [headerTemp, singupTemp] = await Promise.allSettled([
            loadTemplate("/templates/header_unauthorized.html.hbs"),
            loadTemplate("/templates/signup.html.hbs")
        ])
        signupTemplate = singupTemp.value;
        headerTemplate = headerTemp.value;
    };

    const [headerTransl, signUpPageTransl] = await Promise.allSettled([
        fetch(`/lang/${lang}/header.json`).then(r => r.json()),
        fetch(`/lang/${lang}/sign-up.json`).then(r => r.json()),
    ]);
    headerTranslations = headerTransl.value;
    signUpPageTranslations = signUpPageTransl.value;
    document.title = signUpPageTranslations.pageTitle;

    const headerDraw = headerTemplate({ selected_lang: lang.toUpperCase(), lang: headerTranslations });
    const singupDraw = signupTemplate({ lang: signUpPageTranslations });
    document.getElementById("header").innerHTML = headerDraw;
    initDropDowns();

    document.getElementById("signup-box").innerHTML = singupDraw
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

    // Handle language change
    const enEl = document.getElementById("lang-en");
    const ruEl = document.getElementById("lang-ru");
    enEl.onclick = async () => { await handleLangChange("en") };
    ruEl.onclick = async () => { await handleLangChange("ru") };
}

document.addEventListener("DOMContentLoaded", async () => { init(getBrowserLanguage()); });
