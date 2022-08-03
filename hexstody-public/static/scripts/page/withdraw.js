import {
    initTabs,
    formattedCurrencyValue,
    formattedCurrencyValueFixed
}
    from "./common.js";



const refreshInterval = 10000;

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
const ethAddressEl = document.getElementById("eth_address");

const btcValidationDisplayEl = document.getElementById("btc_validation");
const ethValidationDisplayEl = document.getElementById("eth_validation");

async function postWithdrawRequest(currency, address, amount) {
    let body;
    let res;
    switch (currency) {
        case "BTC":
            body = { address: { type: "BTC", addr: address }, amount: amount }
            res = await fetch("/withdraw",
            {
                method: "POST",
                body: JSON.stringify(body)
            });

        return res;
            break;
        case "ETH":
            body = { address: { type: "ETH", account: address }, amount: amount };
            res = await fetch("/withdraweth/" + address + "/" + amount,
                {
                    method: "GET"
                });

            return res;
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
    initTabs(tabs);
    updateLoop();

    Handlebars.registerHelper('currencies', () => ["btc", "eth"]);

    /* maxBtcAmountBtn.onclick = () => btcSendAmountEl.value =
         Math.max(0, btcBalanceEl.getAttribute("balance") - btcFeeEl.getAttribute("fee"));*/

    maxEthAmountBtn.onclick = () => ethSendAmountEl.value =
        Math.max(0, ethBalanceEl.getAttribute("balance") - ethFeeEl.getAttribute("fee"));

    //    ethBalanceEl.innerText = formattedCurrencyValue("ETH",
    //        ethBalanceEl.getAttribute("balance"));
    /*
        
    
       
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
    return await fetch("/ethfee").then(r => r.json());
};

async function updateBalanceAndFeeLoop() {

    const balancesObj = await getBalances();
    var balanceEth;
    balancesObj.balances.forEach(balance => {
        if (balance.currency === "ETH") {
            balanceEth = balance.value;
        }
    });


    const feeObj = await getEthFee();
    const tikerObj = await getCourseForETH("ETH")

    ethBalanceEl.setAttribute("balance", balanceEth);
    const balToUSD = (tikerObj.USD * balanceEth / 1000000000000000000).toFixed(2);
    const txtBal = formattedCurrencyValue("ETH", balanceEth) + " ETH" + " ($ " + balToUSD + ")"
    ethBalanceEl.textContent = txtBal;

    ethFeeEl.setAttribute("fee", feeObj.FastGasPrice);
    const feeToUSD = (tikerObj.USD * 21 * feeObj.FastGasPrice / 1000000).toFixed(2);
    const txtFee = formattedCurrencyValueFixed("ETH", 210000 * feeObj.FastGasPrice * 1000000000, 5) + " ETH" + " ($" + feeToUSD + ")";
    ethFeeEl.textContent = txtFee;
}


async function updateLoop() {
    await Promise.allSettled([updateBalanceAndFeeLoop()]);
    updateBalanceAndFeeLoop();
    await new Promise((resolve) => setTimeout(resolve, refreshInterval));
    updateLoop();
}

async function getCourseForETH(currency) {
    return await fetch("/ethticker",
        {
            method: "POST",
            body: JSON.stringify(currency)
        }).then(r => r.json());
};

document.addEventListener("DOMContentLoaded", init);
