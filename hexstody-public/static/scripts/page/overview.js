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

async function getHistoryERC20(tokenAddr) {
    return fetch("/historyerc20/" + tokenAddr).then(r => r.json());
}

async function getUserData() {
    return fetch("/userdata/").then(r => r.json());
}

async function getHistoryETH() {
    return fetch("/historyeth").then(r => r.json());
}

async function getCourseForBTC(currency) {
    return await fetch("/btcticker",
        {
            method: "POST",
            body: JSON.stringify(currency)
        }).then(r => r.json());
};

async function getCourseForETH(currency) {
    return await fetch("/ethticker",
        {
            method: "POST",
            body: JSON.stringify(currency)
        }).then(r => r.json());
};

async function getCourseForERC20(currency, token) {
    return await fetch("/erc20ticker/" + token,
        {
            method: "GET",
        }).then(r => r.json());
};

async function getBalanceERC20(currency, token) {
    return await fetch("/getBalanceERC20/" + token,
        {
            method: "GET",
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
    Handlebars.registerHelper('formatCurrencyValue', function () {
        if (
            typeof this.currency === 'object'
        ) {
            return formattedCurrencyValue(this.currency.ERC20.ticker, this.value);
        } else {
            return formattedCurrencyValue(this.currency, this.value);
        }
    });

    Handlebars.registerHelper('formatCurrencyName', function () {
        if (
            typeof this.currency === 'object'
        ) {
            return this.currency.ERC20.ticker;
        } else {
            return this.currency;
        }
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

function compareHist(a, b) {
    if (a.timeStamp > b.timeStamp) {
        return -1;
    }
    if (a.timeStamp < b.timeStamp) {
        return 1;
    }
    return 0;
}

async function loadHistoryETH() {
    const userData = await getUserData();
    const histFull = userData.data.historyEth
    for (var i = 0; i < userData.data.tokens.length; i++) {
        histFull.concat(userData.data.historyTokens[i])
    }

    const historyBTCpred = await getHistory(0, 20);
    let histBTCready = [];
    for (var i = 0; i < historyBTCpred.history_items.length; i++) {
        console.log("test " + i + " " + historyBTCpred.history_items.length)
        let htb = historyBTCpred.history_items[i];
        htb.addr = "addrbtc";
        htb.arrow = "mdi-arrow-collapse-down";
        htb.blockNumber = "blockbtc";
        htb.confirmations = historyBTCpred.history_items[i].number_of_confirmations;
        htb.contractAddress = "";
        htb.fee = "250Sat";
        htb.flowType = "Deposit";
        htb.fromtoshow = "genesis";
        htb.fromtoshow = "0x9297db...";
        htb.gas = "0";
        htb.gasPrice = "0";
        htb.hashtoshow = "hashbtc";
        htb.timeStamp = historyBTCpred.history_items[i].date;
        htb.totoshow = "ownbtc";
        htb.tokenName = "BTC";
        htb.valuetoshow = (historyBTCpred.history_items[i].value * 0.00000001).toFixed(8) + " BTC";
        histBTCready.push(htb);
    }

    var hist = { histories: histFull }
    for (var i = 0; i < hist.histories.length; i++) {
        console.log(hist.histories[i].tokenName)
        hist.histories[i].timeStamp = timeStampToTime(parseInt(hist.histories[i].timeStamp));
        hist.histories[i].hashtoshow = hist.histories[i].hash.slice(0, 8) + "...";
        hist.histories[i].fromtoshow = hist.histories[i].from.slice(0, 8) + "...";
        hist.histories[i].totoshow = hist.histories[i].to.slice(0, 8) + "...";
        hist.histories[i].valuetoshow = formattedCurrencyValue(hist.histories[i].tokenName, hist.histories[i].value) + " " + hist.histories[i].tokenName;
        hist.histories[i].fee = formattedCurrencyValue("ETH", hist.histories[i].gas * hist.histories[i].gasPrice);
        if (hist.histories[i].addr.toUpperCase() == hist.histories[i].from.toUpperCase()) {
            hist.histories[i].arrow = "mdi-arrow-collapse-up"
            hist.histories[i].flowType = "Withdraw"
        }
        else {
            hist.histories[i].arrow = "mdi-arrow-collapse-down"
            hist.histories[i].flowType = "Deposit"
        }
    };
    hist.histories = histBTCready.concat(hist.histories)

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
        + "-" + date.getMonth()
        + "-" + date.getDay()
        + " " + hours + ':' + minutes.substr(-2) + ':' + seconds.substr(-2);
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
    const jsonresBTC = await getCourseForETH("BTC")
    const usdToBtc = document.getElementById("usd-BTC");
    let currValBtcPre = document.getElementById("curr-val-BTC").textContent;
    const currValBtc = currValBtcPre.replace(",", "") * 0.00000001;
    document.getElementById("curr-val-BTC").textContent = currValBtc.toFixed(8);
    usdToBtc.textContent = "$" + (currValBtc * jsonresBTC.USD).toFixed(2);

    const jsonres = await getCourseForETH("ETH")
    const usdToEth = document.getElementById("usd-ETH");
    const currValEth = document.getElementById("curr-val-ETH").textContent;
    usdToEth.textContent = "$" + (currValEth * jsonres.USD).toFixed(2);
    const jsonresUSDT = await getCourseForERC20("USDT", "USDT")
    const usdToUSDT = document.getElementById("usd-USDT");
    const currValUSDT = document.getElementById("curr-val-USDT").textContent;
    usdToUSDT.textContent = "$" + (currValUSDT * 1.0).toFixed(2);

    const jsonresCRV = await getCourseForERC20("CRV", "CRV")
    const usdToCRV = document.getElementById("usd-CRV");
    const currValCRV = document.getElementById("curr-val-CRV").textContent;
    usdToCRV.textContent = "$" + (currValCRV * jsonresCRV.USD).toFixed(2);

    const awBal = await (currValCRV * jsonresCRV.USD + parseFloat(currValUSDT) + currValEth * jsonres.USD + currValBtc * jsonresBTC.USD)
    const awBalRub = await (currValCRV * jsonresCRV.RUB + currValUSDT * jsonresUSDT.RUB + currValEth * jsonres.RUB + currValBtc * jsonresBTC.RUB)

    const totalUsd = document.getElementById("total-bal-usd");
    const totalRub = document.getElementById("total-bal-rub");
    totalUsd.textContent = "$" + (awBal).toFixed(2);
    totalRub.textContent = "₽" + (awBalRub).toFixed(2);

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
