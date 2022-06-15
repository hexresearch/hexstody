import { initTabs, formattedCurrencyValue } from "./common.js";

const btcBalanceEl = document.getElementById("btc_balance");
const ethBalanceEl = document.getElementById("eth_balance");

const btcFeeEl = document.getElementById("btc_fee");
const ethFeeEl = document.getElementById("eth_fee");

const btcSendAmountEl = document.getElementById("btc_send_amount");
const ethSendAmountEl = document.getElementById("eth_send_amount")

async function init() {
    const tabs = ["btc-tab", "eth-tab"];
    initTabs(tabs);

    btcBalanceEl.innerText = formattedCurrencyValue("BTC",
        btcBalanceEl.getAttribute("balance"));

    ethBalanceEl.innerText = formattedCurrencyValue("ETH",
        ethBalanceEl.getAttribute("balance"));

    document.getElementById("max_btc").onclick = () =>
        btcSendAmountEl.value =
        btcBalanceEl.getAttribute("balance") - btcFeeEl.getAttribute("fee");

    document.getElementById("max_eth").onclick = () =>
        ethSendAmountEl.value =
        ethBalanceEl.getAttribute("balance") - ethFeeEl.getAttribute("fee");
}

document.addEventListener("DOMContentLoaded", init);