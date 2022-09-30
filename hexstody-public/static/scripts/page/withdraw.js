import {
    initTabs,
    currencyPrecision,
    convertToSmallest,
    currencyNameToCurrency,
    formattedCurrencyValue,
    feeCurrency,
    isErc20Token,
    ETH_TX_GAS_LIMIT,
    ERC20_TX_GAS_LIMIT,
    GWEI
} from "../common.js"

import { localizeSpan } from "../localize.js"


var tabs = []
const refreshInterval = 3_000_000
let withdrawTranslations
let network

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

function validateBtcAmout(amount) {
    let result = {
        ok: true,
        error: null,
        value: null
    }
    let value = Number(amount)
    if (isNaN(value) || value <= 0 || !Number.isInteger(value)) {
        result.ok = false
        result.error = withdrawTranslations.error.invalidAmount
    } else {
        result.value = value
    }
    return result
}

function validateEthAmount(currency, amount) {
    let result = {
        ok: true,
        error: null,
        value: null
    }
    let value = Number(amount)
    if (isNaN(value) || value <= 0) {
        result.ok = false
        result.error = withdrawTranslations.error.invalidAmount
    } else {
        result.value = Math.round(convertToSmallest(currency, value))
    }
    return result
}

function validateAmount(currency, amount) {
    switch (currency.toUpperCase()) {
        case "BTC":
            return validateBtcAmout(amount)
        case "ETH":
        case "USDT":
        case "CRV":
        case "GTECH":
            return validateEthAmount(currency, amount)
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
    const amountValidationResult = validateAmount(currency, amount)
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
        try {
            const responseJson = await response.json()
            if (!response.ok) {
                otherErrorEl.innerText = `Error: ${responseJson.message}`
                otherErrorEl.style.display = "block"
            }
        } catch {
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

async function getFee(currencyName) {
    let fee
    switch (currencyName) {
        case "btc":
            // amount of fee in satoshi
            fee = await fetch("/btcfee").then(r => r.json())
            return fee
        case "eth":
            fee = await fetch("/ethfee").then(r => r.json())
            return fee.ProposeGasPrice * GWEI * ETH_TX_GAS_LIMIT
        case "usdt":
        case "crv":
        case "gtech":
            fee = await fetch("/ethfee").then(r => r.json())
            return fee.ProposeGasPrice * GWEI * ERC20_TX_GAS_LIMIT
    };
}

async function getCurrencyExchangeRate(currency) {
    return await fetch("/ticker/ticker",
        {
            method: "POST",
            body: JSON.stringify(currency)
        }).then(r => r.json())
};

function calcAvailableBalance(balanceObj) {
    const lim = balanceObj.limit_info.limit.amount
    const spent = balanceObj.limit_info.spent
    const value = balanceObj.value
    if (value < (lim - spent)) {
        return value
    } else {
        return (lim - spent)
    };
}

function withdrawalUnits(currency) {
    switch (currency) {
        case "btc":
            return "sat"
        default:
            return currency.toUpperCase()
    }
}

function cryptoToFiat(currencyName, value, rate) {
    // This means ticker is not available
    if (!rate || 'code' in rate) {
        return "-"
    };
    const val = value * rate.USD / currencyPrecision(currencyName)
    const numberFormat = Intl.NumberFormat('ru-RU', {
        style: 'currency',
        currency: 'USD'
    })
    return numberFormat.format(val)
}

async function updateActiveTab() {
    const activeTab = document.querySelector(`#tabs-ul li.is-active`)
    const activeCurrencyName = activeTab.id.replace("-tab", "")
    const currencyNameUppercase = activeCurrencyName.toUpperCase()
    const currency = currencyNameToCurrency(currencyNameUppercase)
    const balanceObj = await getBalance(currencyNameToCurrency(activeCurrencyName))
    const fee = await getFee(activeCurrencyName)
    // GTECH is not listed on any exchange for now
    let tikerResponse
    if (activeCurrencyName === "gtech") {
        tikerResponse = null
    } else {
        tikerResponse = await getCurrencyExchangeRate(currency)
    };
    let feeCurrencyTickerResponse
    // For ERC20 tokens fee is paid in ETH
    if (isErc20Token(activeCurrencyName)) {
        feeCurrencyTickerResponse = await getCurrencyExchangeRate(feeCurrency(activeCurrencyName))
    } else {
        feeCurrencyTickerResponse = tikerResponse
    };

    const availableBalance = calcAvailableBalance(balanceObj)

    const fiatAvailableBalance = cryptoToFiat(activeCurrencyName, availableBalance, tikerResponse)
    const fiatFee = cryptoToFiat(feeCurrency(activeCurrencyName), fee, feeCurrencyTickerResponse)
    const fiatLimit = cryptoToFiat(activeCurrencyName, balanceObj.limit_info.limit.amount, tikerResponse)
    const fiatSpent = cryptoToFiat(activeCurrencyName, balanceObj.limit_info.spent, tikerResponse)

    const availableBalanceElement = document.getElementById(`${activeCurrencyName}-balance`)
    availableBalanceElement.innerHTML = `${formattedCurrencyValue(currencyNameUppercase, availableBalance)} ${currencyNameUppercase} (${fiatAvailableBalance})`

    const feeElement = document.getElementById(`${activeCurrencyName}-fee`)
    feeElement.innerHTML = `${formattedCurrencyValue(feeCurrency(activeCurrencyName), fee)} ${feeCurrency(activeCurrencyName)} (${fiatFee})`

    const limitElement = document.getElementById(`${activeCurrencyName}-limit`)
    limitElement.innerHTML = `${formattedCurrencyValue(currencyNameUppercase, balanceObj.limit_info.limit.amount)} ${currencyNameUppercase} (${fiatLimit}) / ${localizeSpan(balanceObj.limit_info.limit.span)}`

    const spentElement = document.getElementById(`${activeCurrencyName}-spent`)
    spentElement.innerHTML = `${formattedCurrencyValue(currencyNameUppercase, balanceObj.limit_info.spent)} ${currencyNameUppercase} (${fiatSpent})`

    const unitsElement = document.getElementById(`${activeCurrencyName}-units`)
    unitsElement.innerHTML = withdrawalUnits(activeCurrencyName)

    const maxAmountBtn = document.getElementById(`max-${activeCurrencyName}`)
    const sendBtn = document.getElementById(`send-${activeCurrencyName}`)
    const sendAmountInput = document.getElementById(`${activeCurrencyName}-send-amount`)
    const addressInput = document.getElementById(`${activeCurrencyName}-address`)

    maxAmountBtn.onclick = () => {
        if (isErc20Token(activeCurrencyName)) {
            sendAmountInput.value = availableBalance
        } else {
            sendAmountInput.value = Math.max(0, availableBalance - fee)
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
    await updateActiveTab()
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
