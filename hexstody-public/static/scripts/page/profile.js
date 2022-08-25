import { loadTemplate, initTabs, initCollapsibles, getUserName, chunkifyTransposed, indexArrayFromOne } from "./common.js";
import { localizeChangeStatus, localizeSpan, getLanguage } from "./localize.js";
import { hasKeyPairStored, generateKeyPair, keyToMnemonic, storeKeyPair, retrievePrivateKey, removeStoredKeyPair } from "../crypto.js";

const errorBox = document.getElementById("error-box")

var tabs = [];
let tokensTemplate = null;
let limitsTemplate = null;
let settingsTemplate = null
let limitsDict = null;
let tokensDict = null;
let settingsDict = null;
let origLimits = null;
let origConfig = null;
let keyTemplate = null;
let mnemonicTemplate = null;

const emailRegex = /^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})/;
const phoneRegex = /^(\+\d{1,2}\s?)?1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$|^(\+\d{1,2}\s?)?1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{2}[\s.-]?\d{2}$/;

async function getTokens() {
    return await fetch("/profile/tokens/list").then(r => r.json());
};

async function getLimits(){
    return await fetch("/profile/limits/get").then(r => r.json());
}

async function getMyChanges(){
    return await fetch("/profile/limits/changes").then(r => r.json())
}

async function getMyConfig(){
    return await fetch("/profile/settings/config").then(r => r.json())
}

async function postConfigChanges(body){
    console.log(body)
    return await fetch("/profile/settings/config",
    {
        method: "POST",
        body: JSON.stringify(body)
    });
}

async function postEnable(token) {
    const body = {token: token}
    return await fetch("/profile/tokens/enable",
    {
        method: "POST",
        body: JSON.stringify(body)
    });
}

async function postDisable(token) {
    const body = {token: token}
    return await fetch("/profile/tokens/disable",
    {
        method: "POST",
        body: JSON.stringify(body)
    });
}

async function postLimitsChange(changes){
    return await fetch("/profile/limits",
    {
        method: "POST",
        body: JSON.stringify(changes)
    });
}

async function postPasswordChange(oldPass, newPass){
    const body = {old_password: oldPass, new_password: newPass}
    return await fetch("/password",
    {
        method: "POST",
        body: JSON.stringify(body)
    });
}

async function postChangeCancel(currency){
    return await fetch("/profile/limits/cancel",
    {
        method: "POST",
        body: JSON.stringify(currency)
    });
}

function postPublicKeyDer(keyPair, password){
    return async function(){
        const pubDer = await keyPair.publicKey.export('der');
        const encPubDer = Base64.fromUint8Array(pubDer)
        const username = getUserName();
        const resp = await fetch("/profile/key",
        {
            method: "POST",
            body: JSON.stringify(encPubDer)
        });
        if (resp.ok){
            await storeKeyPair(username, password, keyPair)
            loadKeyTab()
        }
    }
}

async function clearPublicKey(){
    const username = getUserName();
    removeStoredKeyPair(username);
    const resp = await fetch("/profile/key",
        {
            method: "POST",
            body: null
        });
    if (resp.ok){
        loadKeyTab()
    }
}

function changeCancelHandler(currency){
    return async function(){
        await postChangeCancel(currency);
        loadLimits();
    }
}

function getCurName(val){
    if (val.currency == null) {
        return "null"
    } else if (typeof val.currency === 'object') {
        return val.currency[Object.keys(val.currency)[0]].ticker
    } else {
        return val.currency
    }
}

async function initTemplates() {
    const [tokensTemp, tokensD, limitsTemp, limitsD, settingsTemp, settingsD, keyTemp, mnemTemp] = await Promise.allSettled([
        await loadTemplate("/templates/token.html.hbs"),
        await fetch("/lang/token.json").then(r => r.json()),
        await loadTemplate("/templates/limits.html.hbs"),
        await fetch("/lang/limits.json").then(r => r.json()),
        await loadTemplate("/templates/settings.html.hbs"),
        await fetch("/lang/settings.json").then(r => r.json()),
        await loadTemplate("/templates/key.html.hbs"),
        await loadTemplate("/templates/mnemonic.html.hbs")
    ]);

    tokensTemplate = tokensTemp.value;
    limitsTemplate = limitsTemp.value;
    settingsTemplate = settingsTemp.value
    limitsDict = limitsD.value;
    tokensDict = tokensD.value;
    settingsDict = settingsD.value;
    keyTemplate = keyTemp.value;
    mnemonicTemplate = mnemTemp.value;
    Handlebars.registerHelper('tokenFormatName', function () { return this.token.ticker; });
    Handlebars.registerHelper('limitsFormatName', function () { return getCurName(this) });
    Handlebars.registerHelper('changesFormatName', function () { return getCurName(this) });
    Handlebars.registerHelper('changeStatus', function () { return localizeChangeStatus(this.status) });
    Handlebars.registerHelper('localizeSpan', function () { return localizeSpan(this.limit.span)})
}

function setLimit(limit){
    const cur = getCurName(limit);
    const span = limit.limit_info.limit.span;
    const value = limit.limit_info.limit.amount;
    const valEl = document.getElementById(cur + "-value");
    const spanEl = document.getElementById(cur + "-span");
    valEl.value = value; 
    spanEl.value = span;
}

async function checkboxHandler(event, token) {
    if (event.currentTarget.checked) {
        const resp = await postEnable(token);
        loadTokens();
    } else {
        const resp = await postDisable(token);
        loadTokens();
    }
}

async function loadTokens() {
    const tokens = await getTokens();
    const tokensDrawUpdate = tokensTemplate({tokens: tokens.tokens, lang: tokensDict});
    const tokensElem = document.getElementById("tokens-tab-body");
    tokensElem.innerHTML = tokensDrawUpdate;
    const tokensArray = tokens.tokens;
    tokensArray.forEach(token => {
        const checkbox = document.getElementById('checkbox-'+token.token.ticker)
        checkbox.addEventListener('change', (event) => checkboxHandler(event, token.token))
    });
}

async function loadLimits(){
    const limits = await getLimits();
    const changes = await getMyChanges();
    origLimits = limits;
    const limitsDrawUpdate = limitsTemplate({limits: limits, changes: changes, lang: limitsDict});
    const limitsElem = document.getElementById("limits-tab-body");
    limitsElem.innerHTML = limitsDrawUpdate;
    limits.forEach(limit => { 
        setLimit(limit) 
        setChangeHandlers(getCurName(limit))
    });

    changes.forEach(change => {
        const btn = document.getElementById(change.id + "-btn");
        btn.onclick = changeCancelHandler(change.currency)
    })
}

async function loadSettings(editable, load){
    hideError()
    if (load){
        origConfig = await getMyConfig()
    }
    const settingsDrawUpdate = settingsTemplate({config: origConfig, lang: settingsDict, editable: editable});
    const settingsElement = document.getElementById("settings-tab-body");
    settingsElement.innerHTML = settingsDrawUpdate

    if (editable){
        const confirmBtn = document.getElementById("confirm-config-changes-btn");
        const clearBtn = document.getElementById("reset-changes-btn");
        confirmBtn.onclick = confirmCfgChanges;
        clearBtn.onclick = loadSettingsCurry(false, false);
    } else {
        const editBtn = document.getElementById("edit-config-btn");
        editBtn.onclick = loadSettingsCurry(true, false);
    }
}

function displayError(errMsg){
    errorBox.style.display = "block";
    errorBox.getElementsByTagName("span")[0].innerText = errMsg;
}

function hideError(){
    errorBox.style.display = "none";
}

async function confirmCfgChanges(){
    hideError()
    const emailVal = document.getElementById("cfg-email").value;
    const phoneVal = document.getElementById("cfg-phone").value;
    const tgnickVal = document.getElementById("cfg-tgnick").value;
    const lang = getLanguage()
    if(!(emailRegex.test(emailVal)) & emailVal !== ""){
        switch (lang){
            case "en": displayError("Invalid e-mail"); break
            case "ru": displayError("Некорректный e-mail"); break
        }
        return
    }
    if(!(phoneRegex.test(phoneVal)) & phoneVal !== ""){
        switch (lang){
            case "en": displayError("Invalid phone number"); break
            case "ru": displayError("Некорректный номер телефона"); break
        }
        return
    }

    var changedEmail = null;
    var changedPhone = null;
    var changedTgnick = null;
    if (origConfig.email){
        if (origConfig.email !== emailVal){
            changedEmail = emailVal    
        }
    } else {
        if (emailVal.length != 0){
            changedEmail = emailVal    
        }
    }
    if (origConfig.phone){
        if (origConfig.phone !== phoneVal){
            changedPhone = phoneVal    
        }
    } else {
        if (phoneVal.length != 0){
            changedPhone = phoneVal    
        }
    }
    if (origConfig.tg_name){
        if (origConfig.tg_name !== tgnickVal){
            changedTgnick = tgnickVal    
        }
    } else {
        if (tgnickVal.length != 0){
            changedTgnick = tgnickVal    
        }
    }
    if (changedEmail != null || changedPhone != null || changedTgnick != null){
        const body = {
            email: changedEmail,
            phone: changedPhone,
            tg_name: changedTgnick
        }
        const res = await postConfigChanges(body);
        if(!res.ok){
            const txt = await res.json();
            displayError(txt.message)
        } else {
            loadSettings(false, true)
        }
    } else {
        loadSettings(false, false)
    }
}

function loadSettingsCurry(editable, load){
    return function(){ return loadSettings(editable, load) }
}

function setChangeHandlers(cur) {
    const valEl = document.getElementById(cur + "-value");
    const spanEl = document.getElementById(cur + "-span");
    valEl.oninput = checkLimitsChange;
    spanEl.addEventListener("change", (event) => checkLimitsChange());
}

function commitChanges(changes){
    return async function(){
        await postLimitsChange(changes)
        loadLimits()
    }
}

function checkLimitsChange() {
    var changes = [];
    origLimits.forEach(ol => {
        const cur = getCurName(ol);
        const valEl = document.getElementById(cur + "-value");
        const spanEl = document.getElementById(cur + "-span");
        if (!(valEl.value == ol.limit_info.limit.amount && spanEl.value == ol.limit_info.limit.span)){
            changes.push({
                currency: ol.currency,
                limit: {
                    amount: parseInt(valEl.value),
                    span: spanEl.value
                }
            })
        }
    })
    const commitBtn = document.getElementById("commit-limits-btn");
    const clearBtn = document.getElementById("clear-changes-btn");
    if (changes.length !== 0) {
        commitBtn.style.display = 'inline-flex'
        commitBtn.onclick = commitChanges(changes);
        clearBtn.style.display = 'inline-flex';
        clearBtn.onclick = loadLimits;
    } else {
        commitBtn.style.display = 'none'
        commitBtn.onclick = commitChanges(changes);
        clearBtn.style.display = 'none';
        clearBtn.onclick = loadLimits;
    }
}

async function performPasswordChange(){
    hideError()
    const oldPassEl = document.getElementById("old-password");
    const oldPass = oldPassEl.value;
    const newPassEl = document.getElementById("new-password");
    const newPass = newPassEl.value;
    const newPassRepEl = document.getElementById("new-password-rep");
    const newPassRep = newPassRepEl.value;

    if (newPass !== newPassRep){
        displayError("Passwords do not match")
    } else {
        const resp = await postPasswordChange(oldPass, newPass);
        if(resp.ok) {
            oldPassEl.value = "";
            newPassEl.value = "";
            newPassRepEl.value = "";
            document.getElementById("pass-change-label-btn").click();
        } else {
            const errMsg = await resp.json();
            displayError(errMsg)
        }
    }
}

async function displayMnemonic(privateKey){
    const mnemonic = await keyToMnemonic(privateKey)
    const chunks = chunkifyTransposed(indexArrayFromOne(mnemonic), 4)
    const mnemDraw = mnemonicTemplate({chunks:chunks})
    document.getElementById("mnemonic-display").innerHTML = mnemDraw;
    document.getElementById("mnemonic-password-box").style.display = "none";
    document.getElementById("mnemonic-box").style.display = "block";
}

async function genMnemonic(){
    hideError()
    const mnemPass = document.getElementById("gen-mnemonic-password").value;
    const mnemPassRep = document.getElementById("gen-mnemonic-password-rep").value;
    if (mnemPass != mnemPassRep) {
        displayError("Passwords do not match!")
    } else {
        const key = await generateKeyPair()
        displayMnemonic(key.privateKey)
        const submitBtn = document.getElementById("set-key");
        submitBtn.onclick = postPublicKeyDer(key, mnemPass);
    }
}

async function showMnemonic(){
    const username = getUserName()
    const password = document.getElementById("show-mnemonic-password").value;
    const keyPair = await retrievePrivateKey(username, password)
    displayMnemonic(keyPair.privateKey)
    const clearBtn = document.getElementById("clear-key");
    clearBtn.onclick = clearPublicKey;
}

async function loadKeyTab(){
    const name = getUserName();
    const hasKey = hasKeyPairStored(name);
    const keyDrawUpdate = keyTemplate({hasKey: hasKey});
    const keyElement = document.getElementById("key-tab-body");
    keyElement.style.width = "100%"
    keyElement.innerHTML = keyDrawUpdate
    initCollapsibles()
    document.getElementById("password-change-btn").onclick = performPasswordChange;
    if (hasKey){
        const showMnemBtn = document.getElementById("show-mnemonic-btn");
        showMnemBtn.onclick = showMnemonic;
    } else {
        const genMnemBtn = document.getElementById("gen-mnemonic-btn");
        genMnemBtn.onclick = genMnemonic;
    }
}

async function tabUrlHook(tabId){
    const tab = tabId.replace("-tab","")
    const name = document.getElementById(tabId).getElementsByTagName("a")[0].innerText.toLowerCase();
    const properName = name.charAt(0).toUpperCase() + name.slice(1);
    const titleEl = document.getElementsByTagName("title")[0];
    const titleOld = titleEl.innerText.split(":");

    window.history.pushState("", "", `/profile?tab=${tab}`);
    titleEl.innerText = titleOld[0] + ": " + properName;

    switch(tabId){
        case "tokens-tab": 
            await loadTokens();
            break;
        case "limits-tab":
            await loadLimits();
            break;
        case "settings-tab":
            await loadSettings(false, true);
            break;
        case "key-tab":
            await loadKeyTab();
            break;
    }
}

function preInitTabs(){
    var selectedIndex = 0;
    const tabEls = document.getElementById("tabs-ul").getElementsByTagName("li");
    for(let i = 0; i < tabEls.length; i++)
    {
        tabs.push(tabEls[i].id);
    }
    const selectedTab = document.getElementById("tabs-ul").getElementsByClassName("is-active");
    if (selectedTab.length != 0) {
        selectedIndex = tabs.indexOf(selectedTab[0].id)
    }
    return selectedIndex
}

async function init() {
    await initTemplates();
    const selectedTab = preInitTabs();
    initTabs(tabs, tabUrlHook, selectedTab);
};

document.addEventListener("headerLoaded", init);
