import { loadTemplate, initTabs } from "./common.js";
import { localizeChangeStatus, localizeSpan, getLanguage } from "./localize.js";

var tabs = [];
let tokensTemplate = null;
let limitsTemplate = null;
let settingsTemplate = null
let limitsDict = null;
let tokensDict = null;
let settingsDict = null;
let origLimits = null;
let origConfig = null;

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

async function postChangeCancel(currency){
    return await fetch("/profile/limits/cancel",
    {
        method: "POST",
        body: JSON.stringify(currency)
    });
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
    const [tokensTemp, tokensD, limitsTemp, limitsD, settingsTemp, settingsD] = await Promise.allSettled([
        await loadTemplate("/templates/token.html.hbs"),
        await fetch("/lang/token.json").then(r => r.json()),
        await loadTemplate("/templates/limits.html.hbs"),
        await fetch("/lang/limits.json").then(r => r.json()),
        await loadTemplate("/templates/settings.html.hbs"),
        await fetch("/lang/settings.json").then(r => r.json()),
    ]);

    tokensTemplate = tokensTemp.value;
    limitsTemplate = limitsTemp.value;
    settingsTemplate = settingsTemp.value
    limitsDict = limitsD.value;
    tokensDict = tokensD.value;
    settingsDict = settingsD.value;
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
    const errEl = document.getElementById("error-box");
    errEl.style.display = "block";
    errEl.getElementsByTagName("span")[0].innerText = errMsg;
}

function hideError(){
    document.getElementById("error-box").style.display = "none";
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
