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

export function initTabs(tabIds) {
    function tabClicked(clickedTabId) {
        tabIds.forEach(tabId => {
            const validationDisplay = document.getElementById(tabId + "-body");
            if (tabId === clickedTabId) {
                document.getElementById(tabId).classList.add("active");
                validationDisplay.style.display = "block";
            } else {
                document.getElementById(tabId).classList.remove("active");
                validationDisplay.style.display = "none";
            }
        });
    }
    tabIds.forEach(tab => document.getElementById(tab).onclick = () => tabClicked(tab));

    function openTab(evt, tabName) {
        var i, x, tablinks;
        x = document.getElementsByClassName("content-tab");
        for (i = 0; i < x.length; i++) {
            x[i].style.display = "none";
        }
        tablinks = document.getElementsByClassName("tab");
        for (i = 0; i < x.length; i++) {
            tablinks[i].className = tablinks[i].className.replace(" is-active", "");
        }
        document.getElementById(tabName).style.display = "block";
        evt.currentTarget.className += " is-active";
    }
}
