export async function getBalance(currency) {
    return fetch("balance", { method: "POST", body: JSON.stringify(currency) }).then(r => r.json())
}