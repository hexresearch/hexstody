import { loadTemplate, formattedCurrencyValue } from "./common.js";

let balanceTemplate = null;
let historyTemplate = null;

async function getBalances() {
    return await fetch("/get_balance").then(r => r.json());
};

async function getHistory(start, amount) {
    return fetch("/get_history").then(r => r.json());
}

async function initTemplates() {
    const [balanceTemp, historyTemp] = await Promise.allSettled([
        await loadTemplate("/static/templates/balance.html.hbs"),
        await loadTemplate("/static/templates/history.html.hbs")
    ]);

    balanceTemplate = balanceTemp.value;
    historyTemplate = historyTemp.value;

    Handlebars.registerHelper('isDeposit', (historyItem) => historyItem.type === "deposit");
    Handlebars.registerHelper('formatCurrencyValue', function() {
        return formattedCurrencyValue(this.currency, this.value);
    });
}

async function updateLoop() {
    const [balance, history] = await Promise.allSettled([
        getBalances(),
        getHistory(0, 100)
    ]);
    const balanceDrawUpdate = balanceTemplate(balance.value);
    const balanceElem = document.getElementById("balance");
    balanceElem.innerHTML = balanceDrawUpdate;

    const historyDrawUpdate = historyTemplate(history.value);
    const historyElem = document.getElementById("history");
    historyElem.innerHTML = historyDrawUpdate;

    await new Promise((resolve) => setTimeout(resolve, 3000));

    updateLoop();
}

async function init() {
    await initTemplates();
    updateLoop();
};

document.addEventListener("DOMContentLoaded", init);