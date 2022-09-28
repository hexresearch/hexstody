import { tickerEnum, formattedCurrencyValue, currencyPrecision, currencyNameToCurrency } from "../common.js";
import { getBalance, postOrderExchange } from "../request.js";

let currencyFrom = null;
let currencyTo = null;
let valueFrom = null;
let valueTo = null;

function displayError(error) {
    const validationDisplay = document.getElementById("validation-error");
    validationDisplay.getElementsByTagName("span")[0].textContent = error;
    validationDisplay.hidden = false;
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
    const ticker = await fetch(`https://min-api.cryptocompare.com/data/price?fsym=${from}&tsyms=${to}`)
        .then(r => r.json());
    const tickerNorm = ticker[to] * currencyPrecision(to) / currencyPrecision(from);
    return Math.round(amount * tickerNorm);
}

function initDrop(idPostfix, options) {
    document.getElementById(`currency-${idPostfix}`).innerHTML = options;
    const optionElements = Array
        .from(document.getElementById(`currency-${idPostfix}`)
            .getElementsByClassName("dropdown-item"));

    for (const opt of optionElements) {
        opt.addEventListener("click", async event => {
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
                    const convertedAmount = await convertAmount(currencyFrom, currencyTo, availableBalance);
                    const formattedAmount = formattedCurrencyValue(currencyTo, convertedAmount);
                    document.getElementById("to_max").innerText = `Max ${formattedAmount}`;
                }
            }
        });
    }

}

async function tryTrans(from, to, event) {
    if (from && to) {
        const inputValue = event.target.value;
        const valueFrom = parseInput(from, inputValue);
        if (valueFrom) {
            return convertAmount(from, to, valueFrom);
        }
    }
    return null;
}

async function init() {
    const allCurrencies = Object.values(tickerEnum);
    const optionTemplate = Handlebars.compile('<a href="#" class="dropdown-item"> {{this}} </a>');
    const renderedOptions = allCurrencies.reduce((acc, opt) => acc + optionTemplate(opt), "");

    document.getElementById("from_value").value = 0;
    document.getElementById("to_value").value = 0;

    document.getElementById("from_value").addEventListener("keyup", async event => {
        valueTo = await tryTrans(currencyFrom, currencyTo, event);
        if (valueTo) {
            document.getElementById("to_value").value = formattedCurrencyValue(currencyTo, valueTo);
        }
    });

    document.getElementById("to_value").addEventListener("keyup", async event => {
        valueFrom = await tryTrans(currencyTo, currencyFrom, event);
        if (valueFrom) {
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

document.addEventListener("DOMContentLoaded", init);