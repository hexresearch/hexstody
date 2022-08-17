import { loadTemplate, formattedCurrencyValue, formattedElapsedTime } from "./common.js";

let balanceTemplate = null;
let historyTemplate = null;
let dict = null;
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
    return await fetch("/ticker",
        {
            method: "POST",
            body: JSON.stringify(currency)
        }).then(r => r.json());
};

async function getCourseForETH(currency) {
    return await fetch("/ticker",
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

    const [balanceTemp, historyTemp, dictTemp] = await Promise.allSettled([
        await loadTemplate("/templates/balance.html.hbs"),
        await loadTemplate("/templates/history.html.hbs"),
        await fetch("/lang/overview-extra.json").then(r => r.json())
    ]);

    balanceTemplate = balanceTemp.value;
    historyTemplate = historyTemp.value;
    dict = dictTemp.value;

    Handlebars.registerHelper('isDeposit', (historyItem) => historyItem.type === "deposit");
    Handlebars.registerHelper('isWithdrawal', (historyItem) => historyItem.type === "withdrawal");
    Handlebars.registerHelper('formatCurrencyValue', function () {
        if (
            typeof this.currency === 'object'
        ) {
            return formattedCurrencyValue(this.currency.ERC20.ticker, this.value);
        } else {
            return formattedCurrencyValue(this.currency, this.value);
        }
    });
    Handlebars.registerHelper('currencyFullName', (currencyName) => {
        switch (currencyName) {
            case "BTC":
                return "Bitcoin";
            case "ETH":
                return "Ethereum";
            default:
                return currencyName;
        }
    })
    Handlebars.registerHelper('formatCurrencyName', function () {
        if (
            typeof this.currency === 'object'
        ) {
            return this.currency.ERC20.ticker;
        } else {
            return this.currency;
        }
    });
    Handlebars.registerHelper('truncate', (text, n) => text.slice(0, n) + "...");

    Handlebars.registerHelper('formattedElapsedTime', function () {
        return formattedElapsedTime(this.date);
    });
    Handlebars.registerHelper('isInProgress', (req_confirmations, confirmations) => req_confirmations > confirmations);
}

async function loadBalance() {
    const balances = await getBalances();
    const balanceDrawUpdate = balanceTemplate({balances: balances.balances, lang: dict});
    const balancesElem = document.getElementById("balances");
    balancesElem.innerHTML = balanceDrawUpdate;
    enableDepositWithdrawBtns(balancesElem);
}

async function loadHistory() {
    const history = await getHistory(0, historyPagesToLoad * historyPageSize - 1);
    const historyDrawUpdate = historyTemplate({history: history.history, lang: dict});
    const historyElem = document.getElementById("history-table");
    historyElem.innerHTML = historyDrawUpdate;
    enableCopyBtns(historyElem);
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
        let htb = historyBTCpred.history_items[i];
        htb.timeStamp = timeStampToTime(Math.round(Date.parse(htb.date) / 1000));
        var isDeposit = htb.type == "deposit";
        var sign = isDeposit ? '+' : '-';
        htb.valuetoshow = sign + formattedCurrencyValue(htb.currency, htb.value) + " " + htb.currency;
        htb.hash = isDeposit ? htb.txid.txid : dict.txdoesntexist;
        htb.explorerLink = isDeposit ? "https://mempool.space/tx/" + htb.txid.txid : "";
        htb.flowClass = isDeposit ? 'is-deposit' : 'is-withdrawal';
        if (!isDeposit) {
            htb.arrow = "mdi-arrow-up";
            htb.flowType = dict.withdraw + " " + htb.currency
        }
        else {
            htb.arrow = "mdi-arrow-collapse-down"
            htb.flowType = dict.deposit + " " + htb.currency
        }
        histBTCready.push(htb);
    }

    var hist = { histories: histFull }
    for (var i = 0; i < hist.histories.length; i++) {
        hist.histories[i].timeStamp = timeStampToTime(parseInt(hist.histories[i].timeStamp));
        var isDeposit = hist.histories[i].addr.toUpperCase() != hist.histories[i].from.toUpperCase();
        var sign = isDeposit ? '+' : '-';
        hist.histories[i].valuetoshow = sign + formattedCurrencyValue(hist.histories[i].tokenName, hist.histories[i].value) + " " + hist.histories[i].tokenName;
        hist.histories[i].explorerLink = "https://etherscan.io/tx/" + hist.histories[i].hash;
        hist.histories[i].flowClass = isDeposit ? 'is-deposit' : 'is-withdrawal';
        if (!isDeposit) {
            hist.histories[i].arrow = "mdi-arrow-up"
            hist.histories[i].flowType = dict.withdraw + " " + hist.histories[i].tokenName
        }
        else {
            hist.histories[i].arrow = "mdi-arrow-collapse-down"
            hist.histories[i].flowType = dict.deposit + " " + hist.histories[i].tokenName
        }
    };

    hist.histories = histBTCready.concat(hist.histories);
    const historyDrawUpdate = historyTemplate(hist);
    const historyElem = document.getElementById("history-table");
    historyElem.innerHTML = historyDrawUpdate;
    enableCopyBtns(historyElem);
}

function enableCopyBtns(historyElem) {
    let tableRows = historyElem.getElementsByTagName('tr');
    for (var row of tableRows) {
        let txId = row.getElementsByTagName('a')[0].innerHTML;
        let copyBtn = row.getElementsByTagName('button')[0];
        copyBtn.addEventListener("click", () => {
            navigator.clipboard.writeText(txId).then(function () { }, function (err) {
                console.error('Could not copy text: ', err);
            });
        });
        tippy(copyBtn, {
            content: dict.copied,
            trigger: "click",
            hideOnClick: false,
            onShow(instance) {
                setTimeout(() => {
                    instance.hide();
                }, 1000);
            }
        });
    };
}

function enableDepositWithdrawBtns(balancesElem) {
    let balanceItems = balancesElem.getElementsByClassName('balances-item');
    for (var item of balanceItems) {
        let depositBtn = item.getElementsByTagName('button')[0];
        let withdrawBtn = item.getElementsByTagName('button')[1];
        depositBtn.addEventListener("click", () => {
            window.location.href = "/deposit";
        });
        withdrawBtn.addEventListener("click", () => {
            window.location.href = "/withdraw";
        });
    };
}

function timeStampToTime(unix_timestamp) {
    var date = new Date(unix_timestamp * 1000);
    var dateStr = date.getFullYear()
        + "-" + String(date.getMonth() + 1).padStart(2, '0')
        + "-" + String(date.getDate()).padStart(2, '0');
    var timeStr = date.toLocaleTimeString();
    return dateStr + " " + timeStr;
}

async function loadMoreHistory() {
    const history = await getHistory(
        historyPagesToLoad * historyPageSize - 1,
        historyPageSize - 1
    );
    const historyDrawUpdate = historyTemplate(history);
    const historyElem = document.getElementById("history-table");
    historyElem.insertAdjacentHTML('beforeend', historyDrawUpdate);
    enableCopyBtns(historyElem);
    historyPagesToLoad += 1;
}

async function updateLoop() {
    await Promise.allSettled([loadBalance(), loadHistoryETH()]);

    const jsonresBTC = await getCourseForBTC("BTC");
    const usdToBtc = document.getElementById("usd-BTC");
    let currValBtc = document.getElementById("curr-val-BTC").textContent;
    usdToBtc.textContent = "(" + (currValBtc * jsonresBTC.USD).toFixed(2) + " USD)";

    const jsonres = await getCourseForETH("ETH")
    const usdToEth = document.getElementById("usd-ETH");
    const currValEth = document.getElementById("curr-val-ETH").textContent;
    usdToEth.textContent = "(" + (currValEth * jsonres.USD).toFixed(2) + " USD)";
    const jsonresUSDT = await getCourseForERC20("USDT", "USDT")
    const usdToUSDT = document.getElementById("usd-USDT");
    const currValUSDT = document.getElementById("curr-val-USDT").textContent;
    usdToUSDT.textContent = "(" + (currValUSDT * 1.0).toFixed(2) + " USD)";


    const awBal = await (parseFloat(currValUSDT) + currValEth * jsonres.USD + currValBtc * jsonresBTC.USD)
    const awBalRub = await (currValUSDT * jsonresUSDT.RUB + currValEth * jsonres.RUB + currValBtc * jsonresBTC.RUB)

    const totalUsd = document.getElementById("total-balance-usd");
    const totalRub = document.getElementById("total-balance-rub");
    totalUsd.textContent = awBal.toFixed(2) + " $";
    totalRub.textContent = "(" + awBalRub.toFixed(2) + " ₽)";

    await new Promise((resolve) => setTimeout(resolve, refreshInterval));
    updateLoop();
}

async function init() {
    await initTemplates();

    const loadMoreButton = document.getElementById("loadMore");
    loadMoreButton.onclick = loadMoreHistory;
    updateLoop();
};


document.addEventListener("headerLoaded", init);
