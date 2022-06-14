import { initTabs , formattedCurrencyValue } from "./common.js";

async function init() {
    const tabs = ["btc-tab", "eth-tab"];
    initTabs(tabs);
    const btcBalanceEl =  document.getElementById("btc_balance");
    btcBalanceEl.innerText = formattedCurrencyValue("BTC", el.innerText);
    const ethBalanceEl =  document.getElementById("eth_balance");
    ethBalanceEl.innerText = formattedCurrencyValue("ETH", el.innerText);
}

document.addEventListener("DOMContentLoaded", init);