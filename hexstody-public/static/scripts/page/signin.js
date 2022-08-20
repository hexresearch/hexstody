import { hasKeyPairStored, listUsers, retrievePrivateKey } from "../crypto.js";

var emailEl = null;
var passwordEl = null;

var signinTemplate = null;

async function loadTemplate(path) {
    const template = await (await fetch(path)).text();
    return Handlebars.compile(template);
}

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
        //        validationDisplay.textContent = (await signInResult.json()).message;
        validationDisplay.hidden = false;
    };
}

function trySubmitOnEnter(e) {
    if (e.key === "Enter") {
        e.preventDefault();
        trySubmit();
    }
}

function trySubmitPKeyOnEnter(e) {
    if (e.key === "Enter") {
        e.preventDefault();
        loginViaKey();
    }
}

async function loginViaKey(){
    document.getElementById("validationError").hidden = true;
    let usernameEl = document.getElementById("username-selector")
    var username;
    if (usernameEl.tagName === "DIV"){
        username = usernameEl.innerText
    } else {
        username = usernameEl.value
    }
    const password = document.getElementById("signInPassword").value;
    try {
        const keyPair = await retrievePrivateKey(username, password)
        console.log(keyPair)
        console.log("PERFORM AUTH HERE")
    } catch (error){
        const validationDisplay = document.getElementById("validationError");
        validationDisplay.getElementsByTagName("span")[0].textContent = error;
        validationDisplay.hidden = false;
    }
}

async function initTemplates(hasKeyOverride){
    const keys = listUsers()
    const hasKeys = keys.length != 0
    const viaKey = hasKeys && hasKeyOverride;
    const [singinTemp] = await Promise.allSettled([loadTemplate("/templates/signin.html.hbs")])
    signinTemplate = singinTemp.value;
    const singinDraw = signinTemplate({viaKey: viaKey, hasKey: hasKeys, keys: keys});
    document.getElementById("signin-box").innerHTML = singinDraw
    if (viaKey) {
        document.getElementById("signin-login").onclick = async function () {await initTemplates(false)}
        document.getElementById("submit").onclick = loginViaKey
        passwordEl = document.getElementById("signInPassword");
        passwordEl.addEventListener("keyup", trySubmitPKeyOnEnter);
    } else {
        const submitButton = document.getElementById("submit");
        submitButton.onclick = trySubmit;
        emailEl = document.getElementById("signInEmail");
        passwordEl = document.getElementById("signInPassword");
        emailEl.addEventListener("keyup", trySubmitOnEnter);
        passwordEl.addEventListener("keyup", trySubmitOnEnter);
        if (hasKeys){
            document.getElementById("signin-key").onclick = async function () {await initTemplates(true)}
        }
    }

}

async function init() {
    Handlebars.registerHelper('isOneElem', function(){return this.keys.length == 1})
    initTemplates(true);
};

document.addEventListener("DOMContentLoaded", init);
