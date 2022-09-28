export async function getBalance(currency) {
    return fetch("balance", { method: "POST", body: JSON.stringify(currency) });
}

export async function postOrderExchange(request) {
    return fetch("/exchange/order", { method: "POST", body: JSON.stringify(request) });
}