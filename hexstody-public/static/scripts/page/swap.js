import { getAllCurrencies, currencyNameToCurrency, formattedCurrencyValue, currencyPrecision } from "./common.js";
import { getBalance } from "./request.js";

var cFrom = null;
var cTo = null;


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




async function init() {
    const allCurrencies = [...getAllCurrencies()];
    const optionTemplate = Handlebars.compile('<a href="#" class="dropdown-item"> {{this}} </a>');
    const renderedOptions = allCurrencies.reduce((acc, opt) => acc + (optionTemplate(opt)), "");

    document.getElementById("from_value").value = 0;
    document.getElementById("to_value").value = 0;

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
                        cFrom = currency;
                        break;
                    case "to":
                        cTo = currency;
                        break;
                }

                if (cFrom !== null) {
                    const bal = await getBalance(currencyNameToCurrency(cFrom));
                    const balPretty = formattedCurrencyValue(cFrom, calcAvailableBalance(bal));
                    document.getElementById("from_max").innerText = `Max ${balPretty}`;
                    if (cTo !== null) {
                        const ticker = await fetch(`https://min-api.cryptocompare.com/data/price?fsym=${cFrom}&tsyms=${cTo}`).then(r => r.json());
                        const t = formattedCurrencyValue(cTo, calcAvailableBalance(bal) * ticker[cTo]);
                        document.getElementById("to_max").innerText = `Max ${t}`;
                    }
                }
            });
        }

    }

    function tran(c, val, ticker) {
        switch (c) {
            case "BTC":
                return formattedCurrencyValue("BTC", val * ticker * currencyPrecision("BTC"));
            default:
                return val * ticker;
        }
    };

    document.getElementById("from_value").addEventListener("keyup", async event => {
        const val = parseFloat(event.target.value);
        if (val && cFrom && cTo) {
            const ticker = await fetch(`https://min-api.cryptocompare.com/data/price?fsym=${cFrom}&tsyms=${cTo}`)
                .then(r => r.json());
            const to = tran(cFrom, val, ticker[cTo]);
            document.getElementById("to_value").value = to;
        }
    });
    initDrop("from", renderedOptions);
    initDrop("to", renderedOptions);

}

document.addEventListener("DOMContentLoaded", init);