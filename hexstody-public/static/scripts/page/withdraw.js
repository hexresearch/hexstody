import {
    initTabs,
    loadTemplate,
    formattedCurrencyValue,
    formattedCurrencyValueFixed,
}
    from "./common.js";

import {localizeSpan} from "./localize.js";

const refreshInterval = 3000000;
var selectedTab = "btc-tab"
var withdrawTabTemplate = null;
var withdrawDict = null;

async function postWithdrawRequest(currency, address, amount) {
    let body;
    switch (currency) {
        case "BTC":
            body = { address: { type: "BTC", addr: address }, amount: amount }
            break;
        case "ETH":
            body = { address: { type: "ETH", account: address }, amount: amount };
    }

    return await fetch("/withdraw",
        {
            method: "POST",
            body: JSON.stringify(body)
        })
};

async function trySubmit(currency, address, amount, validationDisplayEl) {
    const result = await postWithdrawRequest(currency, address, amount);
    if (result.ok) {
        window.location.href = "/overview";
    } else {
        validationDisplayEl.textContent = (await result.json()).message;
        validationDisplayEl.hidden = false;
    }
}

async function init() {
    const tabs = ["btc-tab", "eth-tab"];
    Handlebars.registerHelper('currencies', () => ["btc", "eth"]);

    const [withdrawTabTemp, withdrawDictTemp] = await Promise.allSettled([
        await loadTemplate("/templates/withdraw-tab.html.hbs"),
        await fetch("/lang/withdraw.json").then(r => r.json())
    ]);

    withdrawDict = withdrawDictTemp.value;
    withdrawTabTemplate = withdrawTabTemp.value;
    initTabs(tabs, updateBalanceAndFeeLoop);
    updateLoop();
}

async function getBalance(currency){
    return await fetch("balance", {method: "POST", body: JSON.stringify(currency)}).then(r => r.json())
}

async function getEthFee() {
    return await fetch("/ethfee").then(r => r.json());
};
async function getBtcFee() {
    return await fetch("/btcfee").then(r => r.json());
};

function calcAvailiableBalance(balanceObj){
    const lim = balanceObj.limit_info.limit.amount;
    const spent = balanceObj.limit_info.spent;
    const value = balanceObj.value;
    if (value < (lim - spent)) {
        return value
    } else {
        return (lim - spent)
    }
}

function btcToFiat(value, rate, decimals){
    const val = (value * rate / 100_000_000);
    if (decimals) {
        return val.toFixed(decimals)
    } else {
        return val
    }
}

function ethToFiat(value, rate, decimals){
    const val = (value * rate / 100_000_000_000_000_0000);
    if (decimals) {
        return val.toFixed(decimals)
    } else {
        return val
    }
}

async function updateBtcTab(){
    const balanceObj = await getBalance("BTC");
    const fee = await getBtcFee();
    const balance = calcAvailiableBalance(balanceObj);
    const tikerObj = await getCourseForCurrency("BTC");
    const balToFiat = btcToFiat(balance, tikerObj.USD, 2);
    const feeToFiat = btcToFiat(fee, tikerObj.USD, 2);
    const limitAmount = btcToFiat(balanceObj.limit_info.limit.amount, 1);
    const spentAmount = btcToFiat(balanceObj.limit_info.spent,1);
    const limitToFiat = btcToFiat(balanceObj.limit_info.limit.amount, tikerObj.USD, 2);
    const spentToFiat = btcToFiat(balanceObj.limit_info.spent, tikerObj.USD, 2);

    let cfg = {
        name: "btc",
        balance: `${formattedCurrencyValue("BTC", balance)} BTC ($ ${balToFiat})`,
        fee: `${formattedCurrencyValueFixed("BTC", fee, 5)} BTC ($ ${feeToFiat})`,
        limit: `${limitAmount} BTC ($ ${limitToFiat})/${localizeSpan(balanceObj.limit_info.limit.span)}`,
        spent: `${spentAmount} BTC ($ ${spentToFiat})`,
        lang: withdrawDict
    }

    const drawUpdate = withdrawTabTemplate(cfg);
    const tabBody = document.getElementById('btc-tab-body');
    tabBody.innerHTML = drawUpdate;

    const maxBtcAmountBtn = document.getElementById("max_btc");
    const sendBtcBtn = document.getElementById("send_btc");
    const btcSendAmountEl = document.getElementById("btc_send_amount");
    const btcValidationDisplayEl = document.getElementById("btc_validation");
    const btcAddressEl = document.getElementById("btc_address");

    maxBtcAmountBtn.onclick = () => btcSendAmountEl.value = Math.max(0, balance - fee);
    sendBtcBtn.onclick = () => trySubmit(
        "BTC",
        btcAddressEl.value,
        Number(btcSendAmountEl.value),
        btcValidationDisplayEl);
}

async function updateEthTab(){
    const balanceObj = await getBalance("ETH");
    const balance = calcAvailiableBalance(balanceObj);
    const feeObj = await getEthFee();
    const tikerObj = await getCourseForCurrency("ETH")
    const balToFiat = ethToFiat(balance, tikerObj.USD, 2);
    const feeToFiat = (tikerObj.USD * 21 * feeObj.FastGasPrice / 1000000).toFixed(2);
    const limitAmount = ethToFiat(balanceObj.limit_info.limit.amount, 1);
    const spentAmount = ethToFiat(balanceObj.limit_info.spent,1);
    const limitToFiat = ethToFiat(balanceObj.limit_info.limit.amount, tikerObj.USD, 2);
    const spentToFiat = ethToFiat(balanceObj.limit_info.spent, tikerObj.USD, 2);

    let cfg = {
        name: "eth",
        balance: `${formattedCurrencyValue("ETH", balance)} ETH ($ ${balToFiat})`,
        fee: `${formattedCurrencyValueFixed("ETH", 210000 * feeObj.FastGasPrice * 1000000000, 5)} ETH ($ ${feeToFiat})`,
        limit: `${limitAmount} ETH ($ ${limitToFiat})/${localizeSpan(balanceObj.limit_info.limit.span)}`,
        spent: `${spentAmount} ETH ($ ${spentToFiat})`,
        lang: withdrawDict
    }

    const drawUpdate = withdrawTabTemplate(cfg);
    const tabBody = document.getElementById('eth-tab-body');
    tabBody.innerHTML = drawUpdate;
    const maxEthAmountBtn = document.getElementById("max_eth");    
    const ethSendAmountEl = document.getElementById("eth_send_amount");
    const sendEthBtn = document.getElementById("send_eth");
    const ethAddressEl = document.getElementById("eth_address");
    const ethValidationDisplayEl = document.getElementById("eth_validation");

    maxEthAmountBtn.onclick = () => ethSendAmountEl.value = Math.max(0, balance - feeObj.FastGasPrice);
    sendEthBtn.onclick = () => trySubmit(
        "ETH",
        ethAddressEl.value,
        Number(ethSendAmountEl.value),
        ethValidationDisplayEl);
}

async function updateBalanceAndFeeLoop(clickedTabId) {
    selectedTab = clickedTabId
    switch (clickedTabId){
        case "btc-tab": await updateBtcTab()
        case "eth-tab": await updateEthTab()
    } 
}

async function updateLoop() {
    await new Promise((resolve) => setTimeout(resolve, refreshInterval));
    await updateBalanceAndFeeLoop(selectedTab);
    updateLoop();
}

async function getCourseForCurrency(currency) {
    return await fetch("/ticker",
        {
            method: "POST",
            body: JSON.stringify(currency)
        }).then(r => r.json());
};

document.addEventListener("headerLoaded", init);
