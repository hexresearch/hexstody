import { loadTemplate} from "./common.js";
let depositTemplate = null;

async function getFor(currency) {
    return await fetch("/deposit",
    {
        method: "POST",
        body: JSON.stringify(currency)
    }).then(r => r.json());
};

async function init() {
    depositTemplate = await loadTemplate("/templates/deposit.html.hbs");
    const result = await getFor("BTC");
    const params = {addresses : [{currency: "BTC", address: result.address}]};
    const depositDrawUpdate = depositTemplate(params);
    const depositElem = document.getElementById("deposit");
    depositElem.insertAdjacentHTML('beforeend', depositDrawUpdate);

    new QRCode(document.getElementById("qrcode"), result.address);
}

document.addEventListener("DOMContentLoaded", init);