import { loadTemplate, formattedCurrencyValue, formattedElapsedTime } from "./common.js";

let balanceTemplate = null;
let historyTemplate = null;
const refreshInterval = 3000;
const historyPageSize = 50;
let historyPagesToLoad = 1;


async function getBalances() {
    return await fetch("/balance").then(r => r.json());
};

async function getHistory(skip, take) {
    return fetch(`/history/${skip}/${take}`).then(r => r.json());
}

async function getHistoryETH() {
    return fetch("/historyeth").then(r => r.json());
}

async function getCourseForETH(currency) {
    return await fetch("/ethticker",
    {
        method: "POST",
        body: JSON.stringify(currency)
    }).then(r => r.json());
};

async function initTemplates() {

    const [balanceTemp, historyTemp] = await Promise.allSettled([
        await loadTemplate("/templates/balance.html.hbs"),
        await loadTemplate("/templates/history.html.hbs")
    ]);

    balanceTemplate = balanceTemp.value;
    historyTemplate = historyTemp.value;


    Handlebars.registerHelper('isDeposit', (historyItem) => historyItem.type === "deposit");
    Handlebars.registerHelper('isWithdrawal', (historyItem) => historyItem.type === "withdrawal");
    Handlebars.registerHelper('formatWithdrawalStatus', (status) => {
        switch (status.type) {
            case "InProgress":
                return "In progress";
            case "Confirmed":
                return "Confirmed";
            case "Rejected":
                return "Rejected";
            default:
                return "Unknown";
        };
    });
    Handlebars.registerHelper('formatCurrencyValue', function () {
        return formattedCurrencyValue(this.currency, this.value);
    });
    Handlebars.registerHelper('formattedElapsedTime', function () {
        return formattedElapsedTime(this.date);
    });
    Handlebars.registerHelper('isInProgress', (req_confirmations, confirmations) => req_confirmations > confirmations);
}

async function loadBalance() {
    const balances = await getBalances();
    const balanceDrawUpdate = balanceTemplate(balances);
    const balanceElem = document.getElementById("balance");
    balanceElem.innerHTML = balanceDrawUpdate;
}

async function loadHistory() {
    const history = await getHistory(0, historyPagesToLoad * historyPageSize - 1);
    const historyDrawUpdate = historyTemplate(history);
    const historyElem = document.getElementById("history");
    historyElem.innerHTML = historyDrawUpdate;
}

async function loadHistoryETH() {
    const history = await getHistoryETH();
    const hist = {histories: history}
    console.log(hist);
    const historyDrawUpdate = historyTemplate(hist);
    const historyElem = document.getElementById("history");
    historyElem.innerHTML = historyDrawUpdate;
}

async function loadMoreHistory() {
    const history = await getHistory(
        historyPagesToLoad * historyPageSize - 1,
        historyPageSize - 1
    );
    const historyDrawUpdate = historyTemplate(history);
    const historyElem = document.getElementById("history");

    historyElem.insertAdjacentHTML('beforeend', historyDrawUpdate);

    historyPagesToLoad += 1;
}

async function updateLoop() {
    await Promise.allSettled([loadBalance(), loadHistoryETH()]);
    const jsonres = await getCourseForETH("ETH")
    const usdToEth = document.getElementById("usd-ETH");
    const currValEth = document.getElementById("curr-val-ETH").textContent;
    usdToEth.textContent = "$"+(currValEth*jsonres.USD).toFixed(2);

    const totalUsd = document.getElementById("total-bal-usd");
    const totalRub = document.getElementById("total-bal-rub");
    totalUsd.textContent = "$"+(currValEth*jsonres.USD).toFixed(2);
    totalRub.textContent = "₽"+(currValEth*jsonres.RUB).toFixed(2);

    await new Promise((resolve) => setTimeout(resolve, refreshInterval));
    updateLoop();
}

async function init() {
    await initTemplates();

    const loadMoreButton = document.getElementById("loadMore");
    loadMoreButton.onclick = loadMoreHistory;
    updateLoop();
};


document.addEventListener("DOMContentLoaded", init);
