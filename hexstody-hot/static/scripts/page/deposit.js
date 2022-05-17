async function getFor(currency) {
    return await fetch("/deposit",
    {
        method: "POST",
        body: JSON.stringify(currency)
    }).then(r => r.json());
};

async function init() {
    const submitButton = document.getElementById("withdrawal");
    const result = await getFor("BTC");
    //submitButton.historyElem.insertAdjacentHTML('beforeend', historyDrawUpdate); 
}

document.addEventListener("DOMContentLoaded", init);