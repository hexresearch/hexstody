import { loadTemplate, formattedCurrencyValue, formattedElapsedTime, currencyNameToCurrency } from "./common.js";

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

async function getUserData() {
    return fetch("/userdata/").then(r => r.json());
}

async function getCurrencyExchangeRate(currency) {
    return await fetch("/ticker",
        {
            method: "POST",
            body: JSON.stringify(currency)
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
    Handlebars.registerHelper('truncate', (text, n) => `${text.slice(0, n)} ...`);

    Handlebars.registerHelper('formattedElapsedTime', function () {
        return formattedElapsedTime(this.date);
    });
    Handlebars.registerHelper('isInProgress', (req_confirmations, confirmations) => req_confirmations > confirmations);
    Handlebars.registerHelper('toLowerCase', (s) => s.toLowerCase());
}

async function loadBalance() {
    const balances = await getBalances();
    const balanceDrawUpdate = balanceTemplate({ balances: balances.balances, lang: dict });
    const balancesElem = document.getElementById("balances");
    balancesElem.innerHTML = balanceDrawUpdate;
}

async function loadHistoryETH() {
    const userData = await getUserData();
    const histFull = userData.data.historyTokens.reduce((acc, curr) => acc.concat(curr.history), userData.data.historyEth);

    const historyBTCpred = await getHistory(0, 20);
    let histBTCready = [];
    for (const htb of historyBTCpred.history_items) {
        htb.timeStamp = timeStampToTime(Math.round(Date.parse(htb.date) / 1000));
        const isDeposit = htb.type == "deposit";
        const sign = isDeposit ? '+' : '-';
        htb.valuetoshow = `${sign}${formattedCurrencyValue(htb.currency, htb.value)} ${htb.currency}`;
        htb.hash = isDeposit ? htb.txid.txid : dict.txdoesntexist;
        htb.explorerLink = isDeposit ? `https://mempool.space/tx/${htb.txid.txid}` : "";
        htb.flowClass = isDeposit ? 'is-deposit' : 'is-withdrawal';
        if (!isDeposit) {
            htb.arrow = "mdi-arrow-up";
            htb.flowType = `${dict.withdraw} ${htb.currency}`;
        }
        else {
            htb.arrow = "mdi-arrow-collapse-down"
            htb.flowType = `${dict.deposit} ${htb.currency}`;
        }
        histBTCready.push(htb);
    }

    const hist = { histories: histFull }
    for (const h of hist.histories) {
        h.timeStamp = timeStampToTime(parseInt(h.timeStamp));
        const isDeposit = h.addr.toUpperCase() != h.from.toUpperCase();
        const sign = isDeposit ? '+' : '-';
        h.valuetoshow = `${sign + formattedCurrencyValue(h.tokenName, h.value)} ${h.tokenName}`;
        h.explorerLink = `https://etherscan.io/tx/${h.hash}`;
        h.flowClass = isDeposit ? 'is-deposit' : 'is-withdrawal';
        if (!isDeposit) {
            h.arrow = "mdi-arrow-up"
            h.flowType = `${dict.withdraw} ${h.tokenName}`;
        }
        else {
            h.arrow = "mdi-arrow-collapse-down"
            h.flowType = `${dict.deposit} ${h.tokenName}`;
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

function timeStampToTime(unix_timestamp) {
    var date = new Date(unix_timestamp * 1000);
    var dateStr = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
    var timeStr = date.toLocaleTimeString();
    return `${dateStr} ${timeStr}`;
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

    const usdNumberFormat1 = Intl.NumberFormat('ru-RU', {
        style: 'currency',
        currency: 'USD',
        currencyDisplay: 'code'
    });

    const jsonresBTC = await getCurrencyExchangeRate(currencyNameToCurrency("BTC"));
    const usdToBtc = document.getElementById("usd-BTC");
    let currValBtc = document.getElementById("curr-val-BTC").textContent;
    usdToBtc.textContent = `(${usdNumberFormat1.format((currValBtc * jsonresBTC.USD))})`

    const jsonres = await getCurrencyExchangeRate(currencyNameToCurrency("ETH"));
    const usdToEth = document.getElementById("usd-ETH");
    const currValEth = document.getElementById("curr-val-ETH").textContent;
    usdToEth.textContent = `(${usdNumberFormat1.format((currValEth * jsonres.USD))})`;
    const jsonresUSDT = await getCurrencyExchangeRate(currencyNameToCurrency("USDT"));
    const usdToUSDT = document.getElementById("usd-USDT");
    const currValUSDT = document.getElementById("curr-val-USDT").textContent;
    usdToUSDT.textContent = `(${usdNumberFormat1.format(currValUSDT)})`;


    const awBal = parseFloat(currValUSDT) + currValEth * jsonres.USD + currValBtc * jsonresBTC.USD;
    const awBalRub = currValUSDT * jsonresUSDT.RUB + currValEth * jsonres.RUB + currValBtc * jsonresBTC.RUB;

    const totalUsd = document.getElementById("total-balance-usd");
    const totalRub = document.getElementById("total-balance-rub");

    const usdNumberFormat2 = Intl.NumberFormat('ru-RU', {
        style: 'currency',
        currency: 'USD'
    });
    const rubNumberFormat = Intl.NumberFormat('ru-RU', {
        style: 'currency',
        currency: 'RUB'
    });

    totalUsd.textContent = `${usdNumberFormat2.format(awBal)}`;
    totalRub.textContent = `(${rubNumberFormat.format(awBalRub)})`;

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
