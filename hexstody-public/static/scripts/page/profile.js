import { loadTemplate, initTabs, initCollapsibles, getUserName, chunkifyTransposed, indexArrayFromOne, convertToUnitJson } from "../common.js";
import { localizeChangeStatus, localizeSpan, getLanguage } from "../localize.js";
import { hasKeyPairStored, generateKeyPair, privateKeyToMnemonic, mnemonicToPrivateKey, retrievePrivateKey, removeStoredKeyPair, storePrivateKey } from "../crypto.js";

const errorBox = document.getElementById("error-box")

var tabs = [];
let currencyTabTemplate = null;
let limitsTemplate = null;
let personalTemplate = null
let origLimits = null
let origConfig = null
let securityTemplate = null
let dict = null
let mnemonicTemplate = null

const emailRegex = /^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})/
const phoneRegex = /^(\+\d{1,2}\s?)?1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$|^(\+\d{1,2}\s?)?1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{2}[\s.-]?\d{2}$/

async function getTokens() {
    return await fetch("/profile/tokens/list").then(r => r.json())
}

async function getUnits() {
    return await fetch("/profile/unit/all").then(r => r.json());
}

async function getLimits(){
    return await fetch("/profile/limits/get").then(r => r.json());
}

async function getMyChanges(filter) {
    var filt = null
    if (filter) { filt = filter } else { filt = "all" }
    return await fetch("/profile/limits/changes?filter=" + filt).then(r => r.json())
}

async function getMyConfig() {
    return await fetch("/profile/settings/config").then(r => r.json())
}

async function postConfigChanges(body) {
    return await fetch("/profile/settings/config",
        {
            method: "POST",
            body: JSON.stringify(body)
        })
}

async function postEnable(token) {
    const body = { token: token }
    return await fetch("/profile/tokens/enable",
        {
            method: "POST",
            body: JSON.stringify(body)
        })
}

async function postDisable(token) {
    const body = { token: token }
    return await fetch("/profile/tokens/disable",
        {
            method: "POST",
            body: JSON.stringify(body)
        })
}

async function postLimitsChange(changes) {
    return await fetch("/profile/limits",
        {
            method: "POST",
            body: JSON.stringify(changes)
        })
}

async function postPasswordChange(oldPass, newPass) {
    const body = { old_password: oldPass, new_password: newPass }
    return await fetch("/password",
        {
            method: "POST",
            body: JSON.stringify(body)
        })
}

async function postChangeCancel(currency) {
    return await fetch("/profile/limits/cancel",
        {
            method: "POST",
            body: JSON.stringify(currency)
        })
}

async function postPublicKeyDer(privateKey, password) {
    const pubDer = await privateKey.export('der', { outputPublic: true })
    const encPubDer = Base64.fromUint8Array(pubDer)
    const username = getUserName()
    const resp = await fetch("/profile/key",
        {
            method: "POST",
            body: JSON.stringify(encPubDer)
        })
    if (resp.ok) {
        await storePrivateKey(username, password, privateKey)
        loadSecurityTab()
    }
}

async function clearPublicKey() {
    const username = getUserName()
    removeStoredKeyPair(username)
    const resp = await fetch("/profile/key",
        {
            method: "POST",
            body: null
        })
    if (resp.ok) {
        loadSecurityTab()
    }
}

async function postUnits(units) {
    return await fetch("/profile/unit/set",
    {
        method: "POST",
        body: JSON.stringify(units)
    });
}

function changeCancelHandler(currency){
    return async function(){
        await postChangeCancel(currency);
        loadLimits();
    }
}

function getCurName(val) {
    if (val.currency == null) {
        return "null"
    } else if (typeof val.currency === 'object') {
        return val.currency[Object.keys(val.currency)[0]].ticker
    } else {
        return val.currency
    }
}

async function initTemplates() {
    const [currencyTemp, limitsTemp, personalTemp, securityTemp, mnemTemp, dictD] = await Promise.allSettled([
        await loadTemplate("/templates/profile/currency.html.hbs"),
        await loadTemplate("/templates/profile/limits.html.hbs"),
        await loadTemplate("/templates/profile/settings.html.hbs"),
        await loadTemplate("/templates/profile/security.html.hbs"),
        await loadTemplate("/templates/profile/mnemonic.html.hbs"),
        await fetch("/translations/profile.json").then(r => r.json()),
    ])

    currencyTabTemplate = currencyTemp.value;
    limitsTemplate = limitsTemp.value;
    personalTemplate = personalTemp.value
    securityTemplate = securityTemp.value;
    mnemonicTemplate = mnemTemp.value;
    dict = dictD.value;
    Handlebars.registerHelper('tokenFormatName', function () { return this.token.ticker; });
    Handlebars.registerHelper('limitsFormatName', function () { return getCurName(this) });
    Handlebars.registerHelper('changesFormatName', function () { return getCurName(this) });
    Handlebars.registerHelper('extractUnitValue', function () { return Object.values(this.unit)[0] });
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
        loadCurrencyTab();
    } else {
        const resp = await postDisable(token);
        loadCurrencyTab();
    }
}

async function loadCurrencyTab() {
    const tokens = await getTokens();
    const units = await getUnits();
    const currencyDrawUpdate = currencyTabTemplate({tokens: tokens.tokens, units: units, lang: dict.currency});
    const currencyEl = document.getElementById("currency-tab-body");
    currencyEl.innerHTML = currencyDrawUpdate;
    initCollapsibles();
    const tokensArray = tokens.tokens;
    tokensArray.forEach(token => {
        const checkbox = document.getElementById('checkbox-' + token.token.ticker)
        checkbox.addEventListener('change', (event) => checkboxHandler(event, token.token))
    });
    var acc = {};
    units.forEach(unit => {
        const select = document.getElementById("unit-select-" + getCurName(unit))
        select.value = Object.values(unit.unit)[0];
        select.addEventListener('change', (event) => unitChangeHandler(select.value, units, unit, acc))
    })
}

async function unitChangeHandler(newVal, origUnits, origUnit, acc) {
    const origVal = Object.values(origUnit.unit)[0];
    const id = getCurName(origUnit)
    if (newVal == origVal) {
        delete acc[id]
    } else {
        acc[id] = newVal
    }

    if (Object.keys(acc).length === 0){
        document.getElementById("btn-box").style.display = "none";
    } else {
        document.getElementById("reset-units-btn").onclick = loadCurrencyTab;
        document.getElementById("confirm-units-btn").onclick = postUnitChanges(acc);
        document.getElementById("btn-box").style.display = "block";
    }
}

function postUnitChanges(acc){
    return async function(){
        var units = [];
        Object.keys(acc).forEach(key => {
            const unit = convertToUnitJson(key, acc[key])
            units.push(unit)
        })
        await postUnits(units);
        loadCurrencyTab()
    }
}

async function loadLimits(){
    const limits = await getLimits();
    const changes = await getMyChanges();
    origLimits = limits;
    const limitsDrawUpdate = limitsTemplate({limits: limits, changes: changes, lang: dict.limits});
    const limitsElem = document.getElementById("limits-tab-body");
    limitsElem.innerHTML = limitsDrawUpdate;
    limits.forEach(limit => { 
        setLimit(limit) 
        setChangeHandlers(getCurName(limit))
    })

    changes.forEach(change => {
        const btn = document.getElementById(change.id + "-btn")
        btn.onclick = changeCancelHandler(change.currency)
    })
}

async function loadPersonalTab(editable, load){
    hideError()
    if (load) {
        origConfig = await getMyConfig()
    }
    const settingsDrawUpdate = personalTemplate({config: origConfig, lang: dict.settings, editable: editable});
    const settingsElement = document.getElementById("personal-tab-body");
    settingsElement.innerHTML = settingsDrawUpdate

    if (editable){
        const confirmBtn = document.getElementById("confirm-config-changes-btn");
        const clearBtn = document.getElementById("reset-changes-btn");
        confirmBtn.onclick = confirmCfgChanges;
        clearBtn.onclick = loadPersonalTabCurry(false, false);
    } else {
        const editBtn = document.getElementById("edit-config-btn");
        editBtn.onclick = loadPersonalTabCurry(true, false);
    }
}

function displayError(errMsg) {
    errorBox.style.display = "block"
    errorBox.getElementsByTagName("span")[0].innerText = errMsg
}

function hideError() {
    errorBox.style.display = "none"
}

async function confirmCfgChanges() {
    hideError()
    const emailVal = document.getElementById("cfg-email").value
    const phoneVal = document.getElementById("cfg-phone").value
    const tgnickVal = document.getElementById("cfg-tgnick").value
    const lang = getLanguage()
    if (!(emailRegex.test(emailVal)) & emailVal !== "") {
        switch (lang) {
            case "en": displayError("Invalid e-mail"); break
            case "ru": displayError("Некорректный e-mail"); break
        }
        return
    }
    if (!(phoneRegex.test(phoneVal)) & phoneVal !== "") {
        switch (lang) {
            case "en": displayError("Invalid phone number"); break
            case "ru": displayError("Некорректный номер телефона"); break
        }
        return
    }

    var changedEmail = null
    var changedPhone = null
    var changedTgnick = null
    if (origConfig.email) {
        if (origConfig.email !== emailVal) {
            changedEmail = emailVal
        }
    } else {
        if (emailVal.length != 0) {
            changedEmail = emailVal
        }
    }
    if (origConfig.phone) {
        if (origConfig.phone !== phoneVal) {
            changedPhone = phoneVal
        }
    } else {
        if (phoneVal.length != 0) {
            changedPhone = phoneVal
        }
    }
    if (origConfig.tg_name) {
        if (origConfig.tg_name !== tgnickVal) {
            changedTgnick = tgnickVal
        }
    } else {
        if (tgnickVal.length != 0) {
            changedTgnick = tgnickVal
        }
    }
    if (changedEmail != null || changedPhone != null || changedTgnick != null) {
        const body = {
            email: changedEmail,
            phone: changedPhone,
            tg_name: changedTgnick
        }
        const res = await postConfigChanges(body)
        if (!res.ok) {
            const txt = await res.json()
            displayError(txt.message)
        } else {
            loadPersonalTab(false, true)
        }
    } else {
        loadPersonalTab(false, false)
    }
}

function loadPersonalTabCurry(editable, load){
    return function(){ return loadPersonalTab(editable, load) }
}

function setChangeHandlers(cur) {
    const valEl = document.getElementById(cur + "-value")
    const spanEl = document.getElementById(cur + "-span")
    valEl.oninput = checkLimitsChange
    spanEl.addEventListener("change", (event) => checkLimitsChange())
}

function commitChanges(changes) {
    return async function () {
        await postLimitsChange(changes)
        loadLimits()
    }
}

function hideAllPasswordFields() {
    const els = document.querySelectorAll("input[type='text']")
    console.log(els.length)
    els.forEach(e => {
        e.setAttribute("type", "password")
    })
}

function checkLimitsChange() {
    var changes = []
    origLimits.forEach(ol => {
        const cur = getCurName(ol)
        const valEl = document.getElementById(cur + "-value")
        const spanEl = document.getElementById(cur + "-span")
        if (!(valEl.value == ol.limit_info.limit.amount && spanEl.value == ol.limit_info.limit.span)) {
            changes.push({
                currency: ol.currency,
                limit: {
                    amount: parseInt(valEl.value),
                    span: spanEl.value
                }
            })
        }
    })
    const commitBtn = document.getElementById("commit-limits-btn")
    const clearBtn = document.getElementById("clear-changes-btn")
    if (changes.length !== 0) {
        commitBtn.style.display = 'inline-flex'
        commitBtn.onclick = commitChanges(changes)
        clearBtn.style.display = 'inline-flex'
        clearBtn.onclick = loadLimits
    } else {
        commitBtn.style.display = 'none'
        commitBtn.onclick = commitChanges(changes)
        clearBtn.style.display = 'none'
        clearBtn.onclick = loadLimits
    }
}

async function performPasswordChange() {
    hideError()
    const oldPassEl = document.getElementById("old-password")
    const oldPass = oldPassEl.value
    const newPassEl = document.getElementById("new-password")
    const newPass = newPassEl.value
    const newPassRepEl = document.getElementById("new-password-rep")
    const newPassRep = newPassRepEl.value

    if (newPass !== newPassRep) {
        displayError("Passwords do not match")
    } else {
        const resp = await postPasswordChange(oldPass, newPass)
        if (resp.ok) {
            oldPassEl.value = ""
            newPassEl.value = ""
            newPassRepEl.value = ""
            document.getElementById("pass-change-label-btn").click()
        } else {
            const errMsg = await resp.json()
            displayError(errMsg)
        }
    }
}

async function displayMnemonic(privateKey) {
    const mnemonic = await privateKeyToMnemonic(privateKey)
    const chunks = chunkifyTransposed(indexArrayFromOne(mnemonic), 4)
    const mnemDraw = mnemonicTemplate({ chunks: chunks })
    document.getElementById("mnemonic-display").innerHTML = mnemDraw
    document.getElementById("mnemonic-password-box").style.display = "none"
    document.getElementById("mnemonic-box").style.display = "block"
}

async function genMnemonic() {
    hideError()
    const mnemPass = document.getElementById("gen-mnemonic-password").value
    const mnemPassRep = document.getElementById("gen-mnemonic-password-rep").value
    if (mnemPass != mnemPassRep) {
        displayError("Passwords do not match!")
    } else {
        const key = await generateKeyPair()
        displayMnemonic(key.privateKey)
        const submitBtn = document.getElementById("set-key")
        submitBtn.onclick = async function () {
            await postPublicKeyDer(key.privateKey, mnemPass)

        }
    }
}

async function showMnemonic() {
    const username = getUserName()
    const password = document.getElementById("show-mnemonic-password").value
    const privateKey = await retrievePrivateKey(username, password)
    displayMnemonic(privateKey)
    const clearBtn = document.getElementById("clear-key")
    clearBtn.onclick = clearPublicKey
}

function checkMnemonic() {
    const input = document.getElementById("mnemonic-input").value
    const restoreBtn = document.getElementById("restore-mnemonic-btn")
    const mnem = input.split(" ")
    if (mnem.length == 24) {
        if (mnem[23] != "") {
            restoreBtn.style.display = 'block'
        } else {
            restoreBtn.style.display = 'none'
        }
    } else {
        restoreBtn.style.display = 'none'
    }
}

function restoreMnemonicBtn(privateKey) {
    return async function () {
        const mnemPass = document.getElementById("mnemonic-restore-password").value
        const mnemPassRep = document.getElementById("mnemonic-restore-password-rep").value
        if (mnemPass != mnemPassRep) {
            displayError("Passwords do not match!")
        } else {
            await postPublicKeyDer(privateKey, mnemPass)
        }
    }
}

async function restoreMnemonic() {
    const input = document.getElementById("mnemonic-input").value
    const mnemonicArray = input.split(" ")
    const res = await mnemonicToPrivateKey(mnemonicArray)
    console.log(res)
    if (res.ok) {
        const privateKey = res.value
        const mnemonic = await privateKeyToMnemonic(privateKey)
        const chunks = chunkifyTransposed(indexArrayFromOne(mnemonic), 4)
        const mnemDraw = mnemonicTemplate({ chunks: chunks })
        document.getElementById("mnemonic-restore-display").innerHTML = mnemDraw
        document.getElementById("mnemonic-restore-input-box").style.display = "none"
        document.getElementById("mnemonic-restore-display-box").style.display = "block"
        document.getElementById("mnemonic-restore-password-box").style.display = "block"
        const submitBtn = document.getElementById("mnemonic-restore-btn")
        submitBtn.onclick = restoreMnemonicBtn(privateKey)
    } else {
        displayError(res.error)
    }
}

async function loadSecurityTab() {
    const name = getUserName()
    const hasKey = hasKeyPairStored(name)
    const keyDrawUpdate = securityTemplate({ hasKey: hasKey, lang: dict.security })
    const securityEl = document.getElementById("security-tab-body")
    securityEl.style.width = "100%"
    securityEl.innerHTML = keyDrawUpdate
    initCollapsibles()
    document.getElementById("password-change-btn").onclick = performPasswordChange
    if (hasKey) {
        const showMnemBtn = document.getElementById("show-mnemonic-btn")
        showMnemBtn.onclick = showMnemonic
    } else {
        document.getElementById("gen-mnemonic-btn").onclick = genMnemonic
        document.getElementById("mnemonic-input").onkeyup = checkMnemonic
        document.getElementById("restore-mnemonic-btn").onclick = restoreMnemonic
    }
    hideAllPasswordFields()
}

async function tabUrlHook(tabId) {
    const tab = tabId.replace("-tab", "")
    const name = document.getElementById(tabId).getElementsByTagName("a")[0].innerText.toLowerCase()
    const properName = name.charAt(0).toUpperCase() + name.slice(1)
    const titleEl = document.getElementsByTagName("title")[0]
    const titleOld = titleEl.innerText.split(":")

    window.history.pushState("", "", `/profile?tab=${tab}`)
    titleEl.innerText = titleOld[0] + ": " + properName

    switch(tabId){
        case "currency-tab": 
            await loadCurrencyTab();
            break;
        case "limits-tab":
            await loadLimits();
            break;
        case "personal-tab":
            await loadPersonalTab(false, true);
            break;
        case "security-tab":
            await loadSecurityTab()
            break
    }
}

function preInitTabs() {
    var selectedIndex = 0
    const tabEls = document.getElementById("tabs-ul").getElementsByTagName("li")
    for (let i = 0; i < tabEls.length; i++) {
        tabs.push(tabEls[i].id)
    }
    const selectedTab = document.getElementById("tabs-ul").getElementsByClassName("is-active")
    if (selectedTab.length != 0) {
        selectedIndex = tabs.indexOf(selectedTab[0].id)
    }
    return selectedIndex
}

async function init() {
    await initTemplates()
    const selectedTab = preInitTabs()
    initTabs(tabs, tabUrlHook, selectedTab)
};

document.addEventListener("headerLoaded", init)
