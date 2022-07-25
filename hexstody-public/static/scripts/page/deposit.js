import { initTabs,loadTemplate} from "./common.js";

let depositTemplate = null;

async function getForBTC(currency) {
    return await fetch("/deposit",
    {
        method: "POST",
        body: JSON.stringify(currency)
    }).then(r => r.json());
};

async function getForETH(currency) {
    return await fetch("/deposit_eth",
    {
        method: "POST",
        body: JSON.stringify(currency)
    }).then(r => r.json());
};


async function init() {
    const tabs = ["btc-tab", "eth-tab"];
    //initTabs(tabs);

    depositTemplate = await loadTemplate("/templates/deposit.html.hbs");
    const resultBTC = await getForBTC("BTC");
    const resultETH = await getForETH("ETH");
    const params = {addresses : [{currency: "BTC", address: resultBTC.address}
                                ,{currency: "ETH", address: resultETH.address}]};
    const depositDrawUpdate = depositTemplate(params);
    const depositElem = document.getElementById("deposit");
    depositElem.insertAdjacentHTML('beforeend', depositDrawUpdate);

    const box = document.getElementById('ETH-tab-body');

    box.style.display = 'none';

    new QRCode(document.getElementById("qrcode-BTC"), resultBTC.address);
    new QRCode(document.getElementById("qrcode-ETH"), resultETH.address);
}


document.addEventListener("DOMContentLoaded", init);
