async function init () {
    let response = await fetch("/get_overview");
    alert(JSON.stringify(response.json()));
};

document.addEventListener("DOMContentLoaded", init);

function getBalances (){

}

function getHistory(start, amount){

}