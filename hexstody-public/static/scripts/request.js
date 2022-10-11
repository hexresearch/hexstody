import { currencyNameToCurrency } from "./common.js";
export async function getBalance(currency) {
    return fetch("balance", { method: "POST", body: JSON.stringify(currency) });
}

export async function postOrderExchange(request) {
    return fetch("/exchange/order", { method: "POST", body: JSON.stringify(request) });
}

export async function getAdjustedRate(from, to) {
    return fetch("/ticker/pair/adjusted",
        {
            method: "POST",
            body: JSON.stringify({ from: currencyNameToCurrency(from), to: currencyNameToCurrency(to) })
        });
}