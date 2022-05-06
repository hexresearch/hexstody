let balanceTemplate = null; 

async function init () {

    balanceTemplate = await (await fetch("/static/templates/balance.html.hbs")).text();
    const b = Handlebars.compile(balanceTemplate);
    balanceTemplate = await (await fetch("/static/templates/history.html.hbs")).text();
    const h = Handlebars.compile(balanceTemplate);
    Handlebars.registerHelper('isDeposit', function (h) {
      return h.type === "deposit";
    });

    const [balance, history] = await Promise.allSettled([getBalances(), getHistory(0,100)]);

    const x = b (balance.value);

    const element = document.getElementById("balance");
    element.innerHTML = x;

    const x1 = h (history.value);

    const element1 = document.getElementById("history");
    element1.innerHTML = x1;

    await new Promise((resolve) => setTimeout(resolve, 3000));
    init();
    
};

document.addEventListener("DOMContentLoaded", init);

async function getBalances () {
  return await fetch("/get_balance").then(r => r.json());
};

async function getHistory(start, amount){
  return fetch("/get_history").then(r => r.json());
}