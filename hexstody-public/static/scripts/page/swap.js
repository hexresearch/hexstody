import { getObjByCurrency, validateAmount, displayUnitTickerAmount } from "../common.js";

let balanceFrom = null;
let balanceTo = null;
let pairRate = null;
let amountFrom = null;
let balances = null;

function displayError(error) {
    const validationDisplay = document.getElementById("validation-error");
    validationDisplay.getElementsByTagName("span")[0].textContent = error;
    validationDisplay.hidden = false;
}

async function getBalances() {
    return await fetch("/balance").then(r => r.json());
}

async function postOrderExchange(request) {
    return fetch("/exchange/order", { method: "POST", body: JSON.stringify(request) });
}

async function getAdjustedRate(from, to) {
    return fetch("/ticker/pair/adjusted",
        {
            method: "POST",
            body: JSON.stringify({ from: from, to: to })
        });
}

function wipeEnv(){
    balances = null;
    balanceFrom = null;
    balanceTo = null;
    pairRate = null;
    amountFrom = null;
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
        balanceFrom = getObjByCurrency(balances.balances, curName)
        if(balanceFrom && balanceTo) {
            if (balanceFrom != balanceTo){
                displayEnv()
            }
        }
    }

    selectTo.onchange = function(){
        balanceTo = null;
        let curName = selectTo.value;
        balanceTo = getObjByCurrency(balances.balances, curName)
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
        amountFrom = val.value
        const wholeUnitsFrom = val.value / balanceFrom.value.prec;
        const wholeUnitsTo = wholeUnitsFrom * pairRate.rate;
        const amountTo = wholeUnitsTo * balanceTo.value.prec;
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
        amountFrom = wholeUnitsFrom * balanceFrom.value.prec
        const displayUnitsFrom = amountFrom / balanceFrom.value.mul
        document.getElementById("from_value").value = displayUnitsFrom
    } else {
        document.getElementById("from_value").value = null
    }
}

async function handleSwapButton(){
    const val = validateAmount(balanceFrom.currency, amountFrom)
    if(val.ok){
        const request = {
            currency_from: balanceFrom.currency,
            currency_to: balanceTo.currency,
            amount_from: val.value
        }
        const result = await postOrderExchange(request);
        if (result.ok) {
            window.location.href = "/overview"
        } else {
            displayError((await result.json()).message);
        }
    } else {
        displayError(val.error)
    }
}

document.addEventListener("DOMContentLoaded", initEnv);