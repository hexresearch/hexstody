import { initTabs, formattedCurrencyValue } from "./common.js";

const btcBalanceEl = document.getElementById("btc_balance");
const ethBalanceEl = document.getElementById("eth_balance");

const btcFeeEl = document.getElementById("btc_fee");
const ethFeeEl = document.getElementById("eth_fee");

const btcSendAmountEl = document.getElementById("btc_send_amount");
const ethSendAmountEl = document.getElementById("eth_send_amount");

const maxBtcAmountBtn = document.getElementById("max_btc");
const maxEthAmountBtn = document.getElementById("max_eth");

const sendBtcBtn = document.getElementById("send_btc");
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

    return await fetch("/withdraw",
        {
            method: "POST",
            body: JSON.stringify(body)
        })
};

async function trySubmit(currency, address, amount, validationDisplayEl){
    const result = await postWithdrawRequest(currency, address, amount);
    if(result.ok){
        window.location.href = "/overview";
    }else{
        validationDisplayEl.textContent = (await result.json()).message;
        validationDisplayEl.hidden = false;
    }
}

async function init() {
    const tabs = ["btc-tab", "eth-tab"];
    initTabs(tabs);

    btcBalanceEl.innerText = formattedCurrencyValue("BTC",
        btcBalanceEl.getAttribute("balance"));

    ethBalanceEl.innerText = formattedCurrencyValue("ETH",
        ethBalanceEl.getAttribute("balance"));

    maxBtcAmountBtn.onclick = () => btcSendAmountEl.value =
        btcBalanceEl.getAttribute("balance") - btcFeeEl.getAttribute("fee");

    maxEthAmountBtn.onclick = () => ethSendAmountEl.value =
        ethBalanceEl.getAttribute("balance") - ethFeeEl.getAttribute("fee");

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

document.addEventListener("DOMContentLoaded", init);