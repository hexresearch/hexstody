import { loadTemplate, formattedCurrencyValue, formattedElapsedTime, currencyNameToCurrency, displayUnitAmount } from "../common.js";

let balanceTemplate = null;
let historyTemplate = null;
let dict = null;
const refreshInterval = 20000;
const historyPageSize = 50;
let historyPagesToLoad = 1;
let currentBalances = null;

async function getBalances() {
    return await fetch("/balance").then(r => r.json());
}

async function getHistory(skip, take, filter) {
    var filt = null
    if (filter) { filt = filter } else {filt = "all"}
    return fetch(`/history/${skip}/${take}?filter=` + filt).then(r => r.json());
}

async function initTemplates() {

    const [balanceTemp, historyTemp, dictTemp] = await Promise.allSettled([
        loadTemplate("/templates/balance.html.hbs"),
        loadTemplate("/templates/history.html.hbs"),
        fetch("/translations/overview-extra.json").then(r => r.json())
    ]);

    balanceTemplate = balanceTemp.value;
    historyTemplate = historyTemp.value;
    dict = dictTemp.value;

    Handlebars.registerHelper('isDeposit', (historyItem) => historyItem.type === "deposit");
    Handlebars.registerHelper('isWithdrawal', (historyItem) => historyItem.type === "withdrawal");
    Handlebars.registerHelper('displayUnitAmount', function () {
        return displayUnitAmount(this.value)
    });
    Handlebars.registerHelper('displayUsdValue', function (){
        if (this.ticker){
            let numberFormat = Intl.NumberFormat('en', {
                style: 'currency',
                currency: 'USD',
                currencyDisplay: 'code',
                maximumFractionDigits: Math.log10(this.value.mul),
            })
            let value = numberFormat.format(this.ticker.USD * this.value.amount / this.value.mul)
            return "(" + value + ")";
        }
    })
    Handlebars.registerHelper('formatCurrencyValue', function () {
        if (typeof this.currency === 'object') {
            return formattedCurrencyValue(this.currency.ERC20.ticker, this.value/1000000000000);
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
        if (typeof this.currency === 'object') {
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
    currentBalances = balances;
    const balanceDrawUpdate = balanceTemplate({ balances: balances.balances, lang: dict });
    const balancesElem = document.getElementById("balances");
    balancesElem.innerHTML = balanceDrawUpdate;
}

async function loadHistory() {
    function mapHistory(historyItem) {
        const isDeposit = historyItem.type == "deposit";
        const timeStamp = timeStampToTime(Math.round(Date.parse(historyItem.date) / 1000));
        const currencyName = typeof historyItem.currency === 'object' ? historyItem.currency.ERC20.ticker : historyItem.currency;
        if (isDeposit) {
            let explorerLink;
            switch (currencyName) {
                case "BTC":
                    explorerLink = `https://mempool.space/tx/${historyItem.txid.txid}`;
                    break;
                default:
                    explorerLink = `https://etherscan.io/tx/${historyItem.txid.txid}`;
                    break;
            }
            return {
                timeStamp: timeStamp,
                valueToShow: `+${formattedCurrencyValue(currencyName, historyItem.value)} ${currencyName}`,
                hash: historyItem.txid.txid,
                explorerLink: explorerLink,
                flowClass: "is-deposit",
                arrow: "mdi-arrow-collapse-down",
                flowType: `${dict.deposit} ${currencyName}`
            }
        } else {
            let explorerLink;
            switch (currencyName) {
                case "BTC":
                    explorerLink = "";
                    break;
                default:
                    explorerLink = `https://etherscan.io/tx/${historyItem.txid.txid}`;
                    break;
            }
            return {
                timeStamp: timeStamp,
                valueToShow: `-${formattedCurrencyValue(currencyName, historyItem.value)} ${currencyName}`,
                hash: historyItem.txid.txid,
                explorerLink: explorerLink,
                flowClass: "is-withdrawal",
                arrow: "mdi-arrow-up",
                flowType: `${dict.withdraw} ${currencyName}`
            }
        }
    }
    const historyBTCpred = await getHistory(0, 20);
    const mappedHistory = historyBTCpred.history_items.map(mapHistory);
    const historyDrawUpdate = historyTemplate({ histories: mappedHistory });
    const historyElem = document.getElementById("history-table");
    historyElem.innerHTML = historyDrawUpdate;
    enableCopyBtns(historyElem);
}

function enableCopyBtns(historyElem) {
    let tableRows = historyElem.getElementsByTagName('tr');
    for (const row of tableRows) {
        const txId = row.getElementsByTagName('a')[0].innerHTML;
        const copyBtn = row.getElementsByTagName('button')[0];
        copyBtn.addEventListener("click", () => navigator.clipboard.writeText(txId).then(() => { }, err =>
            console.error('Could not copy text: ', err)
        ));
        tippy(copyBtn, {
            content: dict.copied,
            trigger: "click",
            hideOnClick: false,
            onShow(instance) {
                setTimeout(() => instance.hide(), 1000);
            }
        });
    };
}

function timeStampToTime(unix_timestamp) {
    const date = new Date(unix_timestamp * 1000);
    const dateStr = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
    const timeStr = date.toLocaleTimeString();
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

function displayTotalBalance(){
    var usdTotal = 0;
    var rubTotal = 0;
    currentBalances.balances.forEach(bal => {
        if (bal.ticker){
            usdTotal += bal.value.amount * bal.ticker.USD / bal.value.mul;
            rubTotal += bal.value.amount * bal.ticker.RUB / bal.value.mul;
        }
    })

    const usdNumberFormat = Intl.NumberFormat('ru-RU', {
        style: 'currency',
        currency: 'USD'
    });
    const rubNumberFormat = Intl.NumberFormat('ru-RU', {
        style: 'currency',
        currency: 'RUB'
    });

    document.getElementById("total-balance-usd").textContent = `${usdNumberFormat.format(usdTotal)}`;
    document.getElementById("total-balance-rub").textContent = `(${rubNumberFormat.format(rubTotal)})`;
}

async function updateLoop() {
    await Promise.allSettled([loadBalance(), loadHistory()]);
    displayTotalBalance()
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
