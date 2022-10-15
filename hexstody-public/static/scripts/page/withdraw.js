import {
    initTabs,
    currencyNameToCurrency,
    isErc20Token,
    displayUnitTickerAmount,
    validateAmount
} from "../common.js"

var tabs = []
const refreshInterval = 3_000_000
let withdrawTranslations
let network
let activeCurrency;
let activeCurrencyName;
let balance;
let fee;

function networkToBtcNetwork() {
    switch (network) {
        case "Mainnet":
            return Bitcoin.networks.mainnet
        case "Testnet":
            return Bitcoin.networks.testnet
        case "Regtest":
            return Bitcoin.networks.regtest
        default:
            return undefined
    }
}

async function postWithdrawRequest(currency, address, amount) {
    let body
    switch (currency.toUpperCase()) {
        case "BTC":
            body = { address: { type: "BTC", addr: address }, amount: amount }
            break
        case "ETH":
            body = { address: { type: "ETH", token: "ETH", account: address }, amount: amount }
            break
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
            }
    }
    return await fetch("/withdraw",
        {
            method: "POST",
            body: JSON.stringify(body)
        })
};

function validateEthAddress(address) {
    let result = {
        ok: true,
        error: null,
        value: null
    }
    if (!Web3.utils.isAddress(address)) {
        result.ok = false
        result.error = withdrawTranslations.error.invalidAddress
    } else {
        result.value = address
    }
    return result
}

function validateBtcAddress(address) {
    let result = {
        ok: true,
        error: null,
        value: null
    }
    try {
        Bitcoin.address.toOutputScript(address, networkToBtcNetwork(network))
        result.value = address
    } catch (e) {
        result.ok = false
        result.error = withdrawTranslations.error.invalidAddress
    }
    return result
}

function validateAddress(currency, address) {
    switch (currency.toUpperCase()) {
        case "BTC":
            return validateBtcAddress(address)
        case "ETH":
        case "USDT":
        case "CRV":
        case "GTECH":
            return validateEthAddress(address)
        default:
            return { ok: false, error: withdrawTranslations.error.unknownCurrency, value: null }
    };
}

async function trySubmit(currency, address, amount) {
    // Address elements
    const addressInput = document.getElementById(`${currency}-address`)
    const addressErrorEl = document.getElementById(`${currency}-address-error`)

    // Amount elements
    const amountInput = document.getElementById(`${currency}-send-amount`)
    const maxAmountBtn = document.getElementById(`max-${currency}`)
    const amountErrorEl = document.getElementById(`${currency}-amount-error`)

    // Other
    const otherErrorEl = document.getElementById(`${currency}-other-error`)

    // Clear address errors
    addressInput.classList.remove("is-danger")
    addressErrorEl.innerText = ""
    addressErrorEl.style.display = "none"

    // Clear aomunt errors
    amountInput.classList.remove("is-danger")
    maxAmountBtn.classList.remove("is-danger", "is-outlined")
    amountErrorEl.innerText = ""
    amountErrorEl.style.display = "none"

    // Clear other errors
    otherErrorEl.innerText = ""
    otherErrorEl.style.display = "none"

    // Address validation
    const addressValidationResult = validateAddress(currency, address)
    if (!addressValidationResult.ok) {
        addressInput.classList.add("is-danger")
        addressErrorEl.innerText = addressValidationResult.error
        addressErrorEl.style.display = "block"
    }

    // Amount validation
    
    const convertedAmount = Math.floor(amount * balance.value.mul);
    const amountValidationResult = validateAmount(currency, convertedAmount)
    if (!amountValidationResult.ok) {
        amountInput.classList.add("is-danger")
        maxAmountBtn.classList.add("is-danger", "is-outlined")
        amountErrorEl.innerText = amountValidationResult.error
        amountErrorEl.style.display = "block"
    }

    // Stop here if validation failed
    if (!addressValidationResult.ok || !amountValidationResult.ok) {
        return
    }

    try {
        const response = await postWithdrawRequest(currency, addressValidationResult.value, amountValidationResult.value)
        if (response.ok) {
            window.location.href = "/overview"
        } else {
            otherErrorEl.innerText = `Error: status code ${response.status}`
            otherErrorEl.style.display = "block"
        }
    } catch (error) {
        otherErrorEl.innerText = `Error: ${error}`
        otherErrorEl.style.display = "block"
    }
}

async function getBalance(currency) {
    return await fetch("balance", { method: "POST", body: JSON.stringify(currency) })
        .then(r => r.json())
}

async function getFee(currency) {
    return await fetch("fee/get?ticker=true", { method: "POST", body: JSON.stringify(currency) })
        .then(r => r.json())
}

async function changeActiveCurrency(currencyName){
    activeCurrency = null;
    activeCurrencyName = currencyName;
    const currencyNameUppercase = currencyName.toUpperCase()
    activeCurrency = currencyNameToCurrency(currencyNameUppercase);
    updateActiveTab()
}

async function updateActiveTab(){
    balance = await getBalance(activeCurrency)
    fee = await getFee(activeCurrency)
    document.getElementById(`${activeCurrencyName}-balance`).innerText = displayUnitTickerAmount(balance);
    document.getElementById(`${activeCurrencyName}-fee`).innerText = displayUnitTickerAmount(fee)
    document.getElementById(`${activeCurrencyName}-units`).innerText = balance.value.name;

    // Copy value object to carry unit info, switch amount to display spent amount
    let tmp = Object.assign({}, balance.value);
    tmp.amount = balance.limit_info.spent;
    tmp.ticker = balance.ticker;

    document.getElementById(`${activeCurrencyName}-spent`).innerText = displayUnitTickerAmount(tmp)

    // Switch amount to display limit
    tmp.amount = balance.limit_info.limit.amount;
    document.getElementById(`${activeCurrencyName}-limit`).innerText = displayUnitTickerAmount(tmp)
    
    const maxAmountBtn = document.getElementById(`max-${activeCurrencyName}`)
    const sendBtn = document.getElementById(`send-${activeCurrencyName}`)
    const sendAmountInput = document.getElementById(`${activeCurrencyName}-send-amount`)
    const addressInput = document.getElementById(`${activeCurrencyName}-address`)

    // Limits are not applied, since over limit spending is possible with operator's approval
    maxAmountBtn.onclick = () => {
        if (isErc20Token(activeCurrencyName)) {
            sendAmountInput.value = balance.value.amount / balance.value.mul;
        } else {
            sendAmountInput.value = Math.max(0, balance.value.amount - fee.amount) / balance.value.mul;
        };
    }
        
    sendBtn.onclick = () => trySubmit(
        activeCurrencyName,
        addressInput.value,
        sendAmountInput.value
    )
}

    
async function updateLoop() {
    await new Promise((resolve) => setTimeout(resolve, refreshInterval))
    await updateActiveTab()
    updateLoop()
}

async function tabUrlHook(tabId) {
    const tab = tabId.replace("-tab", "")
    window.history.pushState("", "", `/withdraw?tab=${tab}`)
    await changeActiveCurrency(tab)
}

function preInitTabs() {
    var selectedIndex = 0
    const tabEls = document.getElementById("tabs-ul").getElementsByTagName("li")
    for (let i = 0; i < tabEls.length; i++) {
        tabs.push(tabEls[i].id)
    }
    const selectedTab = document.getElementById("tabs-ul").getElementsByClassName("is-active")
    if (selectedTab.length != 0) {
        selectedIndex = tabs.indexOf(selectedTab[0].id)
    }
    return selectedIndex
}

async function init() {
    withdrawTranslations = await fetch("/translations/withdraw.json").then(r => r.json())
    network = await fetch("/network").then(r => r.json())
    const selectedTab = preInitTabs()
    initTabs(tabs, tabUrlHook, selectedTab)
    updateLoop()
}

document.addEventListener("headerLoaded", init)