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
    const resultBTC = await getFor("BTC");
//    const resultETH = await getFor("ETH");
    const params = {addresses : [{currency: "BTC", address: resultBTC.address}
                                ,{currency: "ETH", address: resultBTC.address}]};
    const depositDrawUpdate = depositTemplate(params);
    const depositElem = document.getElementById("deposit");
    depositElem.insertAdjacentHTML('beforeend', depositDrawUpdate);

    new QRCode(document.getElementById("qrcode-BTC"), resultBTC.address);
    new QRCode(document.getElementById("qrcode-ETH"), resultBTC.address);
}

document.addEventListener("DOMContentLoaded", init);
