import { initTabs, formattedCurrencyValue } from "./common.js";



const refreshInterval = 10000;

const ethBalanceEl = document.getElementById("eth_balance");

const btcFeeEl = document.getElementById("btc_fee");
const ethFeeEl = document.getElementById("eth_fee");

const btcSendAmountEl = document.getElementById("btc_send_amount");
const ethSendAmountEl = document.getElementById("eth_send_amount");

const maxBtcAmountBtn = document.getElementById("max_btc");
const maxEthAmountBtn = document.getElementById("max_eth");

const sendBtcBtn = document.getElementById("send_btc_eth");
const sendEthBtn = document.getElementById("send_eth");

const btcAddressEl = document.getElementById("btc_address");
const ethAddressEl = document.getElementById("btc_address");

const btcValidationDisplayEl = document.getElementById("btc_validation");
const ethValidationDisplayEl = document.getElementById("eth_validation");

async function postWithdrawRequest(currency, address, amount) {
    let body;
    switch (currency) {
        case "BTC":
            body = { address: { type: "BTC", addr: address }, amount: amount }
            break;
        case "ETH":
            body = { address: { type: "ETH", account: address }, amount: amount }
            break;
    }

    const res = await fetch("/withdraweth/"+address+"/"+amount,
        {
            method: "GET"
        });

    return res;
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
    initTabs(tabs);
    updateLoop();



    ethBalanceEl.innerText = formattedCurrencyValue("ETH",
        ethBalanceEl.getAttribute("balance"));
/*
    maxBtcAmountBtn.onclick = () => btcSendAmountEl.value =
        Math.max(0, btcBalanceEl.getAttribute("balance") - btcFeeEl.getAttribute("fee"));

    maxEthAmountBtn.onclick = () => ethSendAmountEl.value =
        Math.max(0, ethBalanceEl.getAttribute("balance") - ethFeeEl.getAttribute("fee"));
*/
    sendBtcBtn.onclick = () => trySubmit(
        "BTC",
        btcAddressEl.value,
        Number(btcSendAmountEl.value),
        btcValidationDisplayEl);

    sendEthBtn.onclick = () => trySubmit(
        "ETH",
        ethAddressEl.value,
        Number(ethSendAmountEl.value),
        ethValidationDisplayEl);
}

async function getBalances() {
    return await fetch("/balance").then(r => r.json());
};

async function getEthFee() {
    return await fetch("/eth_fee").then(r => r.json());
};

async function updateBalanceAndFeeLoop() {
    console.log("debug loop 1");
    const balancesObj = await getBalances();
    const balancEth = balancesObj.balances[0]
    const withdrawBalanceElem = document.getElementById("withdraw-bal");
    const withdrawFeeElem = document.getElementById("withdraw-fee");
    console.log("debug loop", balancEth);
    const txtBal = formattedCurrencyValue("ETH", balancEth.value) + " ETH"
    const txtFee = formattedCurrencyValue("ETH", 210000*12*1000000000) + " ETH";
    withdrawBalanceElem.textContent = txtBal;
    withdrawFeeElem.textContent = txtFee
}


async function updateLoop() {
    await Promise.allSettled([updateBalanceAndFeeLoop()]);
    updateBalanceAndFeeLoop();
    await new Promise((resolve) => setTimeout(resolve, refreshInterval));
    updateLoop();
}

/*return await fetch('/ethticker',
{
    method: 'get',
    body: JSON.stringify(currency)
}).then(r => r.json());
*/

document.addEventListener("DOMContentLoaded", init);
