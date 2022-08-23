export async function loadTemplate(path) {
    const template = await (await fetch(path)).text();
    return Handlebars.compile(template);
}

export function formattedCurrencyValue(currency, value) {
    switch (currency) {
        case "BTC":
            // const nf = new Intl.NumberFormat('en-US');
            // return nf.format(value);
            const v = value / 100000000
            return v.toFixed(8)
        case "ETH":
            const newv = value / 1000000000000000000
            return newv.toFixed(8);
        case "USDT":
            const newu = value / 1000000
            return newu.toFixed(8);
        case "CRV":
            const newc = value / 1000000000000000000
            return newc.toFixed(8);
        case "TUSDT":
            const newc = value / 1000000000000000000
            return newc.toFixed(8);
        default:
            return value;
    }
}

export function formattedCurrencyValueFixed(currency, value, fixed) {
    switch (currency) {
        case "BTC":
            // const nf = new Intl.NumberFormat('en-US');
            // return nf.format(value);
            const v = value / 100000000
            return v.toFixed(fixed)
        case "ETH":
            const newv = value / 1000000000000000000
            return newv.toFixed(fixed)
        default:
            return value;
    }
}

const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

export function formattedElapsedTime(dateTimeString) {
    const date = new Date(dateTimeString);
    const currentDate = new Date();
    const localOffset = currentDate.getTimezoneOffset() * MINUTE;
    const msElapsed = currentDate - date + localOffset;
    const rtf = new Intl.RelativeTimeFormat('en', {
        numeric: 'auto'
    });
    function fmt(constant, constantString) {
        return rtf.format(-Math.round(msElapsed / constant), constantString);
    }

    if (msElapsed < MINUTE) {
        return fmt(SECOND, "second");
    } else if (msElapsed < HOUR) {
        return fmt(MINUTE, "minute");
    } else if (msElapsed < DAY) {
        return fmt(HOUR, "hour");
    } else if (msElapsed < DAY * 2) {
        return fmt(DAY, "day");
    } else {
        const localDate = date.getTime() - localOffset;
        return new Date(localDate).toLocaleString();
    }
}

export function initTabs(tabIds, hook, selected) {
    function tabClicked(clickedTabId) {
        tabIds.forEach(tabId => {
            const validationDisplay = document.getElementById(tabId + "-body");
            if (tabId === clickedTabId) {
                document.getElementById(tabId).classList.add("is-active");
                validationDisplay.style.display = "block";
            } else {
                document.getElementById(tabId).classList.remove("is-active");
                validationDisplay.style.display = "none";
            }
        });
        if (typeof hook === 'function') {
            hook(clickedTabId)
        }
    }
    tabIds.forEach(tab => document.getElementById(tab).onclick = () => tabClicked(tab));
    var i;
    if (selected) {i = selected} else {i = 0};
    tabClicked(tabIds[i]);
}

export function initCollapsibles(){
    var coll = document.getElementsByClassName("collapsible");
    var i;

    for (i = 0; i < coll.length; i++) {
      coll[i].addEventListener("click", function() {
        this.classList.toggle("active");
        var content = this.nextElementSibling;
        if (content.style.display === "block") {
          content.style.display = "none";
        } else {
          content.style.display = "block";
        }
      });
    }
}

export function getUserName(){
    const el = document.getElementById("navbarlogin")
    if(el) {
        return el.innerText
    } else {
        return "anon"
    }
}

export function chunkify(array, chunkSize){
    var chunks = []
    for (let i = 0; i < array.length; i += chunkSize) {
        const chunk = array.slice(i, i + chunkSize);
        chunks.push(chunk)
    }
    return chunks
}

export function transpose(array){
    var transposed = [];
    if (array.length > 0){
        for (let i = 0; i < array[0].length; i++){
            transposed.push([])
        }

        for(let i = 0; i < array[0].length;i++){
            for(let j=0;j<array.length;j++){
                transposed[j].push(array[i][j])
            }
        }
    }
    return transposed
}

export function chunkifyTransposed(array, chunkSize){
    var res = []
    for (let i = 0; i < chunkSize; i++){
        res.push([])
    }

    for (let i = 0; i < array.length; i += chunkSize) {
        const chunk = array.slice(i, i + chunkSize);
        for (let j = 0 ; j < chunkSize; j++) {
            res[j].push(chunk[j])
        }
    }

    return res
}

export function indexArrayFromOne(array) {
    var res = [];
    for (let i = 0; i < array.length; i++) {
        res.push({ix: i+1, value:array[i]})
    }
    return res
}
