let balanceTemplate = null; 

async function init () {

    balanceTemplate = await (await fetch("/static/templates/balance.html.hbs")).text();
    const b = Handlebars.compile(balanceTemplate);
    const x = b ({
        firstname: "Yehuda",
        lastname: "Katz",
      });

    const element = document.getElementById("balance");
    element.innerHTML = x;
    const [balance, history] = await Promise.allSettled([getBalances(), getHistory(0,100)]);



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