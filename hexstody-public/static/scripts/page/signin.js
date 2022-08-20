import { hasKeyPairStored, listUsers, retrievePrivateKey } from "../crypto.js";

var emailEl = null;
var passwordEl = null;

var signinTemplate = null;

const dict = {
    "en": {
        login: "Login",
        as: "as",
        keypassword: "Key password",
        incorrect: "Incorrecnt login/password",
        loginbtn: "LOG IN",
        vialogin: "or sign in via login/password",
        viakey: "or sign in via stored key",
        signup: "or sign up",
        password: "Password"
    },
    "ru": {
        login: "Войти",
        as: "как",
        keypassword: "Пароль от ключа",
        incorrect: "Неверный логин/пароль",
        loginbtn: "ВОЙТИ",
        vialogin: "войти с помощью логина и пароля",
        viakey: "войти с помощью сохранённого ключа",
        signup: "зарегистрироваться",
        password: "Пароль"
    }
}

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

async function makeSignedRequest(privKey, requestBody, url, method) {
    const privateKeyJwk = await privKey.export('jwk');
    const publicKeyDer = await privKey.export('der', { outputPublic: true, compact: true });
    const full_url = window.location.href + url.replace("signin/","");
    const nonce = Date.now();
    const msg_elements = requestBody ? [full_url, JSON.stringify(requestBody), nonce] : [full_url, nonce];
    const msg = msg_elements.join(':');
    const encoder = new TextEncoder();
    const binaryMsg = encoder.encode(msg);
    const signature = await window.jscec.sign(binaryMsg, privateKeyJwk, 'SHA-256', 'der').catch(error => {
        alert(error);
    });
    const signature_data_elements = [
        Base64.fromUint8Array(signature),
        nonce.toString(),
        Base64.fromUint8Array(publicKeyDer)
    ];
    const signature_data = signature_data_elements.join(':');
    const params = requestBody ?
        {
            method: method,
            body: JSON.stringify(requestBody),
            headers: {
                'Content-Type': 'application/json',
                'Signature-Data': signature_data
            }
        } : {
            method: method,
            headers: {
                'Signature-Data': signature_data
            }
        };
    const response = await fetch(url, params);
    return response;
}

async function performKeyAuth(privateKey, username){
    const challengeResp = await fetch("/signin/challenge/get", {method: "POST", body: JSON.stringify(username)});
    if (challengeResp.ok) {
        const challenge = await challengeResp.json()
        const loginResp = await makeSignedRequest(privateKey, {user: username, challenge: challenge}, "/signin/challenge/redeem", "POST")
        if (loginResp.ok){
            window.location.href = "/overview"
        } else {
            displayErr((await loginResp.json()).message) 
        }
    } else {
        displayErr((await challengeResp.json()).message)
    }
}

function hideError(){
    document.getElementById("validationError").hidden = true;
}

function displayErr(error){
    const validationDisplay = document.getElementById("validationError");
    validationDisplay.getElementsByTagName("span")[0].textContent = error;
    validationDisplay.hidden = false;
}

async function loginViaKey(){
    hideError()
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

        await performKeyAuth(keyPair.privateKey, username)

    } catch (error){ displayErr(error) }
}

async function initTemplates(hasKeyOverride, lang){
    const keys = listUsers();
    const hasKeys = keys.length != 0
    const viaKey = hasKeys && hasKeyOverride;
    const [singinTemp] = await Promise.allSettled([loadTemplate("/templates/signin.html.hbs")])
    signinTemplate = singinTemp.value;
    const singinDraw = signinTemplate({viaKey: viaKey, hasKey: hasKeys, keys: keys, lang: dict[lang]});
    document.getElementById("signin-box").innerHTML = singinDraw
    if (viaKey) {
        document.getElementById("signin-login").onclick = async function () {await initTemplates(false, lang)}
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
            document.getElementById("signin-key").onclick = async function () {await initTemplates(true, lang)}
        }
    }
    document.getElementById("lang-sel").onchange = handleLangChange(hasKeyOverride);
}

function handleLangChange(hasKeyOverride){
    return async function(){
        const lang = document.getElementById("lang-sel").value;
        await initTemplates(hasKeyOverride, lang)
        document.getElementById("lang-sel").value = lang
    }
}

async function init() {
    Handlebars.registerHelper('isOneElem', function(){return this.keys.length == 1})
    initTemplates(true, "en");
};

document.addEventListener("DOMContentLoaded", init);
