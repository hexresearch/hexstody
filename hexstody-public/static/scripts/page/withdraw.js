import {
    initTabs,
    currencyPrecision,
    currencyNameToCurrency,
    formattedCurrencyValue,
    feeCurrency,
    isErc20Token,
    ETH_TX_GAS_LIMIT,
    ERC20_TX_GAS_LIMIT,
    GWEI
} from "./common.js";

import { localizeSpan } from "./localize.js";

var tabs = [];
const refreshInterval = 3_000_000;

async function postWithdrawRequest(currency, address, amount) {
    let body;
    switch (currency) {
        case "BTC":
            body = { address: { type: "BTC", addr: address }, amount: amount }
            break;
        case "ETH":
            body = { address: { type: "ETH", account: address }, amount: amount };
        case "USDT":
        case "CRV":
        case "GTECH":
            body = {
                address: {
                    type: "ERC20",
                    token: currencyNameToCurrency(currency).ERC20,
                    account: {
                        account: address
                    }
                },
                amount: amount
            };
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

async function getBalance(currency) {
    return await fetch("balance", { method: "POST", body: JSON.stringify(currency) })
        .then(r => r.json())
}

async function getFee(currencyName) {
    let fee;
    switch (currencyName) {
        case "btc":
            // amount of fee in satoshi
            fee = await fetch("/btcfee").then(r => r.json());
            return fee;
        case "eth":
            fee = await fetch("/ethfee").then(r => r.json());
            return fee.ProposeGasPrice * GWEI * ETH_TX_GAS_LIMIT;
        case "usdt":
        case "crv":
        case "gtech":
            fee = await fetch("/ethfee").then(r => r.json());
            return fee.ProposeGasPrice * GWEI * ERC20_TX_GAS_LIMIT;
    };
}

async function getCurrencyExchangeRate(currency) {
    return await fetch("/ticker",
        {
            method: "POST",
            body: JSON.stringify(currency)
        }).then(r => r.json());
};

function calcAvailableBalance(balanceObj) {
    const lim = balanceObj.limit_info.limit.amount;
    const spent = balanceObj.limit_info.spent;
    const value = balanceObj.value;
    if (value < (lim - spent)) {
        return value;
    } else {
        return (lim - spent);
    };
}

function cryptoToFiat(currencyName, value, rate) {
    // This means ticker is not available
    if (!rate || 'code' in rate) {
        return "-";
    };
    const val = value * rate.USD / currencyPrecision(currencyName);
    const numberFormat = Intl.NumberFormat('ru-RU', {
        style: 'currency',
        currency: 'USD'
    });
    return numberFormat.format(val);
}

async function updateActiveTab() {
    const activeTab = document.querySelector(`#tabs-ul li.is-active`);
    const activeCurrencyName = activeTab.id.replace("-tab", "");
    const currencyNameUppercase = activeCurrencyName.toUpperCase()
    const currency = currencyNameToCurrency(currencyNameUppercase);
    const balanceObj = await getBalance(currencyNameToCurrency(activeCurrencyName));
    const fee = await getFee(activeCurrencyName);
    // GTECH is not listed on any exchange for now
    let tikerResponse;
    if (activeCurrencyName === "gtech") {
        tikerResponse = null;
    } else {
        tikerResponse = await getCurrencyExchangeRate(currency);
    };
    let feeCurrencyTickerResponse;
    // For ERC20 tokens fee is paid in ETH
    if (isErc20Token(activeCurrencyName)) {
        feeCurrencyTickerResponse = await getCurrencyExchangeRate(feeCurrency(activeCurrencyName));
    } else {
        feeCurrencyTickerResponse = tikerResponse;
    };

    const availableBalance = calcAvailableBalance(balanceObj);

    const fiatAvailableBalance = cryptoToFiat(activeCurrencyName, availableBalance, tikerResponse);
    const fiatFee = cryptoToFiat(feeCurrency(activeCurrencyName), fee, feeCurrencyTickerResponse);
    const fiatLimit = cryptoToFiat(activeCurrencyName, balanceObj.limit_info.limit.amount, tikerResponse);
    const fiatSpent = cryptoToFiat(activeCurrencyName, balanceObj.limit_info.spent, tikerResponse);

    const availableBalanceElement = document.getElementById(`${activeCurrencyName}-balance`);
    availableBalanceElement.innerHTML = `${formattedCurrencyValue(currencyNameUppercase, availableBalance)} ${currencyNameUppercase} (${fiatAvailableBalance})`;

    const feeElement = document.getElementById(`${activeCurrencyName}-fee`);
    feeElement.innerHTML = `${formattedCurrencyValue(feeCurrency(activeCurrencyName), fee)} ${feeCurrency(activeCurrencyName)} (${fiatFee})`;

    const limitElement = document.getElementById(`${activeCurrencyName}-limit`);
    limitElement.innerHTML = `${formattedCurrencyValue(currencyNameUppercase, balanceObj.limit_info.limit.amount)} ${currencyNameUppercase} (${fiatLimit}) / ${localizeSpan(balanceObj.limit_info.limit.span)}`;

    const spentElement = document.getElementById(`${activeCurrencyName}-spent`);
    spentElement.innerHTML = `${formattedCurrencyValue(currencyNameUppercase, balanceObj.limit_info.spent)} ${currencyNameUppercase} (${fiatSpent})`;

    const maxAmountBtn = document.getElementById(`max-${activeCurrencyName}`);
    const sendBtn = document.getElementById(`send-${activeCurrencyName}`);
    const sendAmountInput = document.getElementById(`${activeCurrencyName}-send-amount`);
    const validationDisplayEl = document.getElementById(`${activeCurrencyName}-validation`);
    const addressInput = document.getElementById(`${activeCurrencyName}-address`);

    maxAmountBtn.onclick = () => {
        if (isErc20Token(activeCurrencyName)) {
            sendAmountInput.value = availableBalance;
        } else {
            sendAmountInput.value = Math.max(0, availableBalance - fee);
        };
    };
    sendBtn.onclick = () => trySubmit(
        currencyNameUppercase,
        addressInput.value,
        Number(sendAmountInput.value),
        validationDisplayEl);
}

async function updateLoop() {
    await new Promise((resolve) => setTimeout(resolve, refreshInterval));
    await updateActiveTab();
    updateLoop();
}

async function tabUrlHook(tabId) {
    const tab = tabId.replace("-tab", "");
    window.history.pushState("", "", `/withdraw?tab=${tab}`);
    await updateActiveTab();
}

function preInitTabs() {
    var selectedIndex = 0;
    const tabEls = document.getElementById("tabs-ul").getElementsByTagName("li");
    for (let i = 0; i < tabEls.length; i++) {
        tabs.push(tabEls[i].id);
    }
    const selectedTab = document.getElementById("tabs-ul").getElementsByClassName("is-active");
    if (selectedTab.length != 0) {
        selectedIndex = tabs.indexOf(selectedTab[0].id);
    }
    return selectedIndex;
}

async function init() {
    const selectedTab = preInitTabs();
    initTabs(tabs, tabUrlHook, selectedTab);
    updateLoop();
}

document.addEventListener("headerLoaded", init);
