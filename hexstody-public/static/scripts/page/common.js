const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

// Amount of satoshi in 1 BTC
const BTC_PRECISION = 10 ** 8;
// Amount of wei in 1 ETH
const ETH_PRECISION = 10 ** 18;
const USDT_PRECISION = 10 ** 6;
const CRV_PRECISION = 10 ** 18;
const GTECH_PRECISION = 10 ** 18;

export const GWEI = 10 ** 9;

const currencyBtc = "BTC";
const currencyEth = "ETH";
const currencyUsdt = {
    "ERC20": {
        "ticker": "USDT",
        "name": "USDT",
        "contract": "0xdAC17F958D2ee523a2206206994597C13D831ec7"
    }
};
const currencyCrv = {
    "ERC20": {
        "ticker": "CRV",
        "name": "CRV",
        "contract": "0xD533a949740bb3306d119CC777fa900bA034cd52"
    }
};
const currencyGtech = {
    "ERC20": {
        "ticker": "GTECH",
        "name": "GTECH",
        "contract": "0xf4904d8c93d7Fd72eff00f7F7A1195E9875da4D1"
    }
};

// Gas limit for ETH transfer transaction
export const ETH_TX_GAS_LIMIT = 21_000;
// Gas limit for ERC20 transfer transaction
export const ERC20_TX_GAS_LIMIT = 150_000;

export async function loadTemplate(path) {
    const template = await (await fetch(path)).text();
    return Handlebars.compile(template);
}

export function formattedCurrencyValue(currency, value) {
    let result;
    switch (currency) {
        case "BTC":
            // const nf = new Intl.NumberFormat('en-US');
            // return nf.format(value);
            result = value / BTC_PRECISION;
            return result.toFixed(8)
        case "ETH":
            result = value / ETH_PRECISION;
            return result.toFixed(8);
        case "USDT":
            result = value / USDT_PRECISION;
            return result.toFixed(8);
        case "CRV":
            result = value / CRV_PRECISION;
            return result.toFixed(8);
        case "GTECH":
            result = value / GTECH_PRECISION;
            return result.toFixed(8);
        default:
            return value;
    };
}

export function formattedCurrencyValueFixed(currency, value, fixed) {
    let result;
    switch (currency) {
        case "BTC":
            // const nf = new Intl.NumberFormat('en-US');
            // return nf.format(value);
            result = value / BTC_PRECISION;
            return result.toFixed(fixed);
        case "ETH":
            result = value / ETH_PRECISION;
            return result.toFixed(fixed);
        case "USDT":
            result = value / USDT_PRECISION;
            return result.toFixed(fixed);
        case "CRV":
            result = value / CRV_PRECISION;
            return result.toFixed(fixed);
        case "GTECH":
            result = value / GTECH_PRECISION;
            return result.toFixed(fixed);
        default:
            return value;
    }
}

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
    if (selected) { i = selected } else { i = 0 };
    tabClicked(tabIds[i]);
}

export function currencyNameToCurrency(currencyName) {
    switch (currencyName.toUpperCase()) {
        case "BTC":
            return currencyBtc;
        case "ETH":
            return currencyEth;
        case "USDT":
            return currencyUsdt;
        case "CRV":
            return currencyCrv;
        case "GTECH":
            return currencyGtech;
        default:
            return null;
    }
}

export function currencyPrecision(currencyName) {
    switch (currencyName.toUpperCase()) {
        case "BTC":
            return BTC_PRECISION;
        case "ETH":
            return ETH_PRECISION;
        case "USDT":
            return USDT_PRECISION;
        case "CRV":
            return CRV_PRECISION;
        case "GTECH":
            return GTECH_PRECISION;
        default:
            return null;
    };
}

// The currency in which transaction fees are paid
export function feeCurrency(currencyName) {
    switch (currencyName.toUpperCase()) {
        case "BTC":
            return "BTC";
        case "ETH":
            return "ETH";
        case "USDT":
        case "CRV":
        case "GTECH":
            return "ETH";
        default:
            return null;
    };
}

export function isErc20Token(currencyName) {
    switch (currencyName.toUpperCase()) {
        case "BTC":
        case "ETH":
            return false;
        case "USDT":
        case "CRV":
        case "GTECH":
            return true;
        default:
            return null;
    };
}