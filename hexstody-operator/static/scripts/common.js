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

const tabContentSuffix = "-content";

export function initTabs(tabContainerId) {
    const tabs = document.querySelectorAll(`#${tabContainerId} a`);
    const activeTab = tabs[0];
    const activeTabContent = document.getElementById(activeTab.id + tabContentSuffix);
    let tabContent;
    for (let i = 1; i < tabs.length; i++) {
        tabContent = document.getElementById(tabs[i].id + tabContentSuffix);
        tabContent.style.display = 'none';
    };
    activeTab.classList.add('active');
    activeTabContent.style.display = 'block';
}

export function openTab(tabContainerId, newActiveTabId) {
    const activeTab = document.querySelector(`#${tabContainerId} a.active`);
    const activeTabContent = document.getElementById(activeTab.id + tabContentSuffix);
    const newActiveTab = document.getElementById(newActiveTabId);
    const newActiveTabContent = document.getElementById(newActiveTabId + tabContentSuffix);
    activeTab.classList.remove('active');
    activeTabContent.style.display = 'none';
    newActiveTab.classList.add('active');
    newActiveTabContent.style.display = 'block';
}