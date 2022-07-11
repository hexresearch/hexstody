import { loadTemplate, formattedCurrencyValue, formattedElapsedTime } from "./common.js";

let balanceTemplate = null;
let historyTemplate = null;
const refreshInterval = 20000;
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
    var hist = {histories: history }
    for(var i=0; i<hist.histories.length;i++){
      hist.histories[i].timeStamp=timeStampToTime(parseInt(hist.histories[i].timeStamp));
      hist.histories[i].hashtoshow=hist.histories[i].hash.slice(0, 10)+"...";
      hist.histories[i].fromtoshow=hist.histories[i].from.slice(0, 10)+"...";
      hist.histories[i].totoshow=hist.histories[i].to.slice(0, 10)+"...";
      hist.histories[i].valuetoshow=formattedCurrencyValue("ETH",hist.histories[i].value);
      hist.histories[i].fee=formattedCurrencyValue("ETH",hist.histories[i].gas*hist.histories[i].gasPrice);
      if (hist.histories[i].addr.toUpperCase() == hist.histories[i].from.toUpperCase()){
          hist.histories[i].arrow = "mdi-arrow-collapse-up"
          hist.histories[i].flowType = "Withdraw"
      }
      else{
        hist.histories[i].arrow = "mdi-arrow-collapse-down"
        hist.histories[i].flowType = "Deposit"
      }
    };
    const historyDrawUpdate = historyTemplate(hist);
    const historyElem = document.getElementById("history");
    historyElem.innerHTML = historyDrawUpdate;
}

function timeStampToTime(unix_timestamp) {
  var date = new Date(unix_timestamp * 1000);
  // Hours part from the timestamp
  var hours = date.getHours();
  // Minutes part from the timestamp
  var minutes = "0" + date.getMinutes();
  // Seconds part from the timestamp
  var seconds = "0" + date.getSeconds();

  // Will display time in 10:30:23 format
  var formattedTime = date.getFullYear()
                    +"-"+date.getMonth()
                    +"-"+date.getDay()
                    +" "+hours + ':' + minutes.substr(-2) + ':' + seconds.substr(-2);
  return formattedTime
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
