import { listUsers, retrievePrivateKey } from "../crypto.js";
import { getBrowserLanguage } from "../localize.js";
import { loadTemplate, initDropDowns } from "../common.js";

var emailEl = null;
var passwordEl = null;

var headerTemplate = null;
var signinTemplate = null;

var headerTranslations = null;
var signInPageTranslations = null;

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
    const full_url = window.location.href + url.replace("signin/", "");
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

async function performKeyAuth(privateKey, username) {
    const challengeResp = await fetch("/signin/challenge/get", { method: "POST", body: JSON.stringify(username) });
    if (challengeResp.ok) {
        const challenge = await challengeResp.json()
        const loginResp = await makeSignedRequest(privateKey, { user: username, challenge: challenge }, "/signin/challenge/redeem", "POST")
        if (loginResp.ok) {
            window.location.href = "/overview"
        } else {
            displayErr((await loginResp.json()).message)
        }
    } else {
        displayErr((await challengeResp.json()).message)
    }
}

function hideError() {
    document.getElementById("validationError").hidden = true;
}

function displayErr(error) {
    const validationDisplay = document.getElementById("validationError");
    validationDisplay.getElementsByTagName("span")[0].textContent = error;
    validationDisplay.hidden = false;
}

async function loginViaKey() {
    hideError()
    let usernameEl = document.getElementById("username-selector");
    let username = usernameEl.value;
    const password = document.getElementById("signInPassword").value;
    await retrievePrivateKey(username, password)
        .then(privateKey => performKeyAuth(privateKey, username).catch(err => displayErr(err)))
        .catch(_err => displayErr(signInPageTranslations.incorrect));
}

async function handleLangChange(lang, hasKeyOverride) {
    await initTemplates(hasKeyOverride, lang);
}

async function initTemplates(hasKeyOverride, lang) {
    const keys = listUsers();
    const hasKeys = keys.length != 0;
    const viaKey = hasKeys && hasKeyOverride;
    if (!signinTemplate) {
        const [headerTemp, singinTemp] = await Promise.allSettled([
            loadTemplate("/templates/header_unauthorized.html.hbs"),
            loadTemplate("/templates/signin.html.hbs")
        ]);

        signinTemplate = singinTemp.value;
        headerTemplate = headerTemp.value;
    };

    const [headerTransl, signInPageTransl] = await Promise.allSettled([
        fetch(`/lang/${lang}/header.json`).then(r => r.json()),
        fetch(`/lang/${lang}/sign-in.json`).then(r => r.json()),
    ]);
    headerTranslations = headerTransl.value;
    signInPageTranslations = signInPageTransl.value;
    document.title = signInPageTranslations.pageTitle;

    const headerDraw = headerTemplate({ selected_lang: lang.toUpperCase(), lang: headerTranslations });
    const singinDraw = signinTemplate({ viaKey: viaKey, hasKey: hasKeys, keys: keys, lang: signInPageTranslations });
    document.getElementById("header").innerHTML = headerDraw;
    initDropDowns();

    // Handle language change
    const enEl = document.getElementById("lang-en");
    const ruEl = document.getElementById("lang-ru");
    enEl.onclick = async () => { await handleLangChange("en", hasKeyOverride) };
    ruEl.onclick = async () => { await handleLangChange("ru", hasKeyOverride) };

    document.getElementById("signin-box").innerHTML = singinDraw;
    if (viaKey) {
        document.getElementById("signin-login").onclick = async function () { await initTemplates(false, lang) }
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
        if (hasKeys) {
            document.getElementById("signin-key").onclick = async function () { await initTemplates(true, lang) }
        }
    }
}

window.addEventListener('load', () => { initTemplates(true, getBrowserLanguage()); });