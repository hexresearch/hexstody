import { currencyToCurrencyName, tickerEnum, formattedCurrencyValue, currencyNameToCurrency, validateAmount, displayUnitTickerAmount } from "../common.js";
import { getAdjustedRate, getBalance, postOrderExchange } from "../request.js";

let balanceFrom = null;
let balanceTo = null;
let pairRate = null;
let valueFrom = null;
let valueTo = null;
let balances = null;

function displayError(error) {
    const validationDisplay = document.getElementById("validation-error");
    validationDisplay.getElementsByTagName("span")[0].textContent = error;
    validationDisplay.hidden = false;
}

function currencyPrecision(){
    return 1
}


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

function parseInput(currency, value) {
    switch (currency) {
        case tickerEnum.btc:
            return parseInt(value);
        default:
            const asFloat = parseFloat(value);
            return asFloat ? Math.round(asFloat * currencyPrecision(currency)) : null;
    }
};

async function convertAmount(from, to, amount) {
    const request = {
        from: currencyNameToCurrency(from),
        to: currencyNameToCurrency(to)
    };
    const ticker = await fetch("/ticker/pair", { method: "POST", body: JSON.stringify(request) })
        .then(r => r.json());
    return Math.round(amount / currencyPrecision(from) * ticker.rate * currencyPrecision(to));
}

function displayTicker(ticker) {
    document.getElementById("rate-span").innerText =
        `1 ${currencyToCurrencyName(ticker.from)} = ${ticker.rate} + ${currencyToCurrencyName(ticker.to)}`;
}

function hideTicker() {
    document.getElementById("rate-span").innerText = "";
}

function initDrop(idPostfix, options) {
    document.getElementById(`currency-${idPostfix}`).innerHTML = options;
    const optionElements = Array
        .from(document.getElementById(`currency-${idPostfix}`)
            .getElementsByClassName("dropdown-item"));

    for (const opt of optionElements) {
        opt.addEventListener("click", async event => {
            hideTicker();
            document.getElementById("from_value").value = 0;
            document.getElementById("to_value").value = 0;
            const currency = event.target.innerText;
            document.getElementById(`currency-selection-${idPostfix}`).innerText = currency;

            switch (idPostfix) {
                case "from":
                    currencyFrom = currency;
                    break;
                case "to":
                    currencyTo = currency;
                    break;
            }

            if (currencyFrom) {
                const balance = await getBalance(currencyNameToCurrency(currencyFrom)).then(r => r.json());
                const availableBalance = calcAvailableBalance(balance);
                const formattedBalance = formattedCurrencyValue(currencyFrom, availableBalance);
                document.getElementById("from_max").innerText = `Max ${formattedBalance}`;
                if (currencyTo) {
                    const ticker = await getAdjustedRate(currencyFrom, currencyTo).then(r => r.json());
                    displayTicker(ticker);
                    const convertedAmount = await convertAmount(currencyFrom, currencyTo, ticker.rate, availableBalance);
                    const formattedAmount = formattedCurrencyValue(currencyTo, convertedAmount);
                    document.getElementById("to_max").innerText = `Max ${formattedAmount}`;
                }
            }
        });
    }

}

function tryTrans(from, event) {
    if (from) {
        const inputValue = event.target.value;
        return parseInput(from, inputValue);
    }
    return null;
}

async function getBalances() {
    return await fetch("/balance").then(r => r.json());
}

function getBalanceByCurrency(currencyName){
    for (const balance of balances.balances){
        let cn = currencyToCurrencyName(balance.currency)
        if (cn === currencyName){
            return balance;
        }
    }
}

function wipeEnv(){
    balances = null;
    balanceFrom = null;
    balanceTo = null;
    pairRate = null;
    valueFrom = null;
    valueTo = null;
    document.getElementById("from-max-btn").style.display = "none";
    document.getElementById("from-avaliable").style.display = "none";
    document.getElementById("from-avaliable").hidden = true;
}

async function initEnv(){
    wipeEnv()
    balances = await getBalances()
    const selectFrom = document.getElementById("select-from")
    const selectTo = document.getElementById("select-to")
    
    selectFrom.onchange = function(){
        balanceFrom = null;
        let curName = selectFrom.value;
        balanceFrom = getBalanceByCurrency(curName)
        if(balanceFrom && balanceTo) {
            if (balanceFrom != balanceTo){
                displayEnv()
            }
        }
    }

    selectTo.onchange = function(){
        balanceTo = null;
        let curName = selectTo.value;
        balanceTo = getBalanceByCurrency(curName)
        if(balanceFrom && balanceTo) {
            if (balanceFrom != balanceTo){
                displayEnv()
            }
        }
    }
}

async function displayEnv(){
    pairRate = await getAdjustedRate(balanceFrom.currency, balanceTo.currency).then(r => r.json());
    document.getElementById("from-balance").innerText = displayUnitTickerAmount(balanceFrom);
    document.getElementById("from-unit").innerText = balanceFrom.value.name;
    document.getElementById("to-unit").innerText = balanceTo.value.name;
    document.getElementById("from-avaliable").hidden = false;
    document.getElementById("rate-span").innerText = displayExchangeRate(pairRate)
    document.getElementById("from-max-btn").style.display = "block";
    document.getElementById("from-max-btn").onclick = maxAmountBtn;
    document.getElementById("from_value").onkeyup = fromChangedHandler;
    document.getElementById("to_value").onkeyup = toChangedHandler;
    document.getElementById("swap").onclick = handleSwapButton;
}

function displayExchangeRate(rate){
    if(rate) {
        return `1 ${rate.from} = ${rate.rate} ${rate.to}`
    } else {
        return ""
    }
}

function maxAmountBtn(){
    const fromEl = document.getElementById("from_value") 
    fromEl.value = balanceFrom.value.amount / balanceFrom.value.mul;
    fromEl.onkeyup()
}

function fromChangedHandler(){
    const rawValue = document.getElementById("from_value").value;
    const convertedAmount = Math.floor(rawValue * balanceFrom.value.mul)
    let val = validateAmount(balanceFrom.currency, convertedAmount);
    if (val.ok){
        // Split calc for easy understanding
        const wholeUnitsFrom = val.value / balanceFrom.value.prec;
        const wholeUnitsTo = wholeUnitsFrom * pairRate.rate;
        const amountTo = wholeUnitsTo * balanceTo.prec;
        const displayUnitsTo = amountTo / balanceTo.value.mul;
        document.getElementById("to_value").value = displayUnitsTo

    } else {
        document.getElementById("to_value").value = null
    }
}

function toChangedHandler(){
    const rawValue = document.getElementById("to_value").value;
    const convertedAmount = Math.floor(rawValue * balanceTo.value.mul)
    let val = validateAmount(balanceTo.currency, convertedAmount);
    if (val.ok) {
        // Split calc for easy understanding
        const wholeUnitsTo = val.value / balanceTo.value.prec
        const wholeUnitsFrom = wholeUnitsTo / pairRate.rate
        const displayUnitsFrom = wholeUnitsFrom * balanceFrom.value.prec / balanceFrom.value.mul
        document.getElementById("from_value").value = displayUnitsFrom
    } else {
        document.getElementById("from_value").value = null
    }
}

async function handleSwapButton(){

}

async function init2() {
    const allCurrencies = Object.values(tickerEnum);
    const optionTemplate = Handlebars.compile('<a href="#" class="dropdown-item"> {{this}} </a>');
    const renderedOptions = allCurrencies.reduce((acc, opt) => acc + optionTemplate(opt), "");

    document.getElementById("from_value").value = 0;
    document.getElementById("to_value").value = 0;

    document.getElementById("from_value").addEventListener("keyup", async event => {
        valueFrom = tryTrans(currencyFrom, event);
        if (valueFrom && currencyTo) {
            valueTo = await convertAmount(currencyFrom, currencyTo, valueFrom);
            document.getElementById("to_value").value = formattedCurrencyValue(currencyTo, valueTo);
        }
    });

    document.getElementById("to_value").addEventListener("keyup", async event => {
        valueTo = tryTrans(currencyTo, event);
        if (valueTo && currencyFrom) {
            valueFrom = await convertAmount(currencyTo, currencyFrom, valueTo);
            document.getElementById("from_value").value = formattedCurrencyValue(currencyFrom, valueFrom);
        }
    });

    document.getElementById("swap").addEventListener("click", async _ => {
        if (currencyFrom && currencyTo && valueFrom && valueTo) {
            const request = {
                currency_from: currencyNameToCurrency(currencyFrom),
                currency_to: currencyNameToCurrency(currencyTo),
                amount_from: valueFrom,
                amount_to: valueTo
            }

            const result = await postOrderExchange(request);
            if (result.ok) {
                window.location.href = "/overview"
            } else {
                displayError((await result.json()).message);
            }
        }
    });

    initDrop("from", renderedOptions);
    initDrop("to", renderedOptions);
}

document.addEventListener("DOMContentLoaded", initEnv);