import { loadTemplate, formattedCurrencyValue, formattedElapsedTime } from "./common.js";

let tokensTemplate = null;
const refreshInterval = 20000;

async function getTokens() {
    return await fetch("/profile/tokens/list").then(r => r.json());
};

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

function buildEnabler(token) {
    return async function enabler(){
        console.log("enable " + token.token.tiker)
        const res = await postEnable(token.token);
        loadTokens();
    } 
}

function buildDisabler(token) {
    return async function disabler(){
        console.log("disable " + token.token.ticker)
        const res = await postDisable(token.token);
        loadTokens();
    }
}

async function initTemplates() {

    const [tokensTemp] = await Promise.allSettled([
        await loadTemplate("/templates/token.html.hbs")
    ]);

    tokensTemplate = tokensTemp.value;

    Handlebars.registerHelper('isDeposit', (historyItem) => historyItem.type === "deposit");
    Handlebars.registerHelper('isWithdrawal', (historyItem) => historyItem.type === "withdrawal");
    Handlebars.registerHelper('formatWithdrawalStatus', (status) => {
        switch (status.type) {
            case "InProgress":
                return "In progress";
            case "Confirmed":
                return "Confirmed";
            case "Completed":
                return "Completed";
            case "OpRejected":
                return "Rejected by operators";
            case "NodeRejected":
                return "Rejected by node";
            default:
                return "Unknown";
        };
    });

    Handlebars.registerHelper('formatCurrencyName', function () { return this.token.ticker; });
    Handlebars.registerHelper('truncate', (text, n) => text.slice(0, n) + "...");

    Handlebars.registerHelper('formattedElapsedTime', function () {
        return formattedElapsedTime(this.date);
    });
    Handlebars.registerHelper('isInProgress', (req_confirmations, confirmations) => req_confirmations > confirmations);

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
    const tokensDrawUpdate = tokensTemplate(tokens);
    const tokensElem = document.getElementById("tokens");
    tokensElem.innerHTML = tokensDrawUpdate;
    const tokensArray = tokens.tokens;
    tokensArray.forEach(token => {
        const checkbox = document.getElementById('checkbox-'+token.token.ticker)
        checkbox.addEventListener('change', (event) => checkboxHandler(event, token.token))
    });
}


async function updateLoop() {
    await loadTokens();
    await new Promise((resolve) => setTimeout(resolve, refreshInterval));
    updateLoop();
}

async function init() {
    await initTemplates();
    updateLoop();
};


document.addEventListener("DOMContentLoaded", init);
