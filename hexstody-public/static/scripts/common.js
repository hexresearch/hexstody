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

export const tickerEnum = Object.freeze({
    btc: "BTC",
    eth: "ETH",
    erc20_usdt: "USDT",
    erc20_crv: "CRV",
    erc20_gtech:"GTECH"
});

export const currencyEnum = Object.freeze({
    btc: tickerEnum.btc,
    eth: tickerEnum.eth,
    erc20_usdt: {
        "ERC20": {
            "ticker": tickerEnum.erc20_usdt,
            "name": "USDT",
            "contract": "0xfD8ef4113c5f54BE9Cb103eB437b710b8e1d6885"
        }
    },
    erc20_crv: {
        "ERC20": {
            "ticker": tickerEnum.erc20_crv,
            "name": "CRV",
            "contract": "0x817805F0f818237c73Fde5dEc91dbB650A7E7612"
        }
    },
    erc20_gtech: {
        "ERC20": {
            "ticker": tickerEnum.erc20_gtech,
            "name": "GTECH",
            "contract": "0x866A4Da32007BA71aA6CcE9FD85454fCF48B140c"
        }
    }
});

// Gas limit for ETH transfer transaction
export const ETH_TX_GAS_LIMIT = 21_000;
// Gas limit for ERC20 transfer transaction
export const ERC20_TX_GAS_LIMIT = 150_000;

export async function loadTemplate(path) {
    const template = await (await fetch(path)).text();
    return Handlebars.compile(template);
}

export function formattedCurrencyValue(currency, value) {
    let numberFormat;
    switch (currency) {
        case tickerEnum.btc:
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(BTC_PRECISION),
            });
            return numberFormat.format(value / BTC_PRECISION);
        case tickerEnum.eth:
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(ETH_PRECISION),
            });
            return numberFormat.format(value / ETH_PRECISION);
        case tickerEnum.erc20_usdt:
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(USDT_PRECISION),
            });
            return numberFormat.format(value / USDT_PRECISION);
        case tickerEnum.erc20_crv:
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(CRV_PRECISION),
            });
            return numberFormat.format(value / CRV_PRECISION);
        case tickerEnum.erc20_gtech:
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(GTECH_PRECISION),
            });
            return numberFormat.format(value / GTECH_PRECISION);
        default:
            return value;
    };
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

export function initCollapsibles() {
    console.log("A")
    const cols = document.getElementsByClassName("collapsible")
    console.log(cols)
    for (let col of cols) {
        col.addEventListener("click", function () {
            this.classList.toggle("active");
            const content = this.nextElementSibling;
            if (content.style.display === "block") {
                content.style.display = "none";
            } else {
                content.style.display = "block";
            }
        });
    }
    // .forEach(function (coll) {

    // });
}

export function initDropDowns() {
    var $dropdowns = getAll('.dropdown:not(.is-hoverable)');

    if ($dropdowns.length > 0) {
        $dropdowns.forEach(function ($el) {
            $el.addEventListener('click', function (event) {
                event.stopPropagation();
                $el.classList.toggle('is-active');
            });
        });

        document.addEventListener('click', function (event) {
            closeDropdowns();
        });
    }

    function closeDropdowns() {
        $dropdowns.forEach(function ($el) {
            $el.classList.remove('is-active');
        });
    }

    // Close dropdowns if ESC pressed
    document.addEventListener('keydown', function (event) {
        var e = event || window.event;
        if (e.key === "Escape") {
            closeDropdowns();
        }
    });

    // Functions
    function getAll(selector) {
        return Array.prototype.slice.call(document.querySelectorAll(selector), 0);
    }
}

export function getUserName() {
    const el = document.getElementById("navbarlogin")
    if (el) {
        return el.innerText
    } else {
        return "anon"
    }
}

export function chunkify(array, chunkSize) {
    var chunks = []
    for (let i = 0; i < array.length; i += chunkSize) {
        const chunk = array.slice(i, i + chunkSize);
        chunks.push(chunk)
    }
    return chunks
}

export function transpose(array) {
    var transposed = [];
    if (array.length > 0) {
        for (let i = 0; i < array[0].length; i++) {
            transposed.push([])
        }

        for (let i = 0; i < array[0].length; i++) {
            for (let j = 0; j < array.length; j++) {
                transposed[j].push(array[i][j])
            }
        }
    }
    return transposed
}

export function chunkifyTransposed(array, chunkSize) {
    var res = []
    for (let i = 0; i < chunkSize; i++) {
        res.push([])
    }

    for (let i = 0; i < array.length; i += chunkSize) {
        const chunk = array.slice(i, i + chunkSize);
        for (let j = 0; j < chunkSize; j++) {
            res[j].push(chunk[j])
        }
    }

    return res
}

export function indexArrayFromOne(array) {
    var res = [];
    for (let i = 0; i < array.length; i++) {
        res.push({ ix: i + 1, value: array[i] })
    }
    return res
}

export function currencyPrecision(currencyName) {
    switch (currencyName.toUpperCase()) {
        case tickerEnum.btc:
            return BTC_PRECISION;
        case tickerEnum.eth:
            return ETH_PRECISION;
        case tickerEnum.erc20_usdt:
            return USDT_PRECISION;
        case tickerEnum.erc20_crv:
            return CRV_PRECISION;
        case tickerEnum.erc20_gtech:
            return GTECH_PRECISION;
        default:
            return null;
    };
}

// The currency in which transaction fees are paid
export function feeCurrency(currencyName) {
    switch (currencyName.toUpperCase()) {
        case tickerEnum.btc:
            return tickerEnum.btc;
        case tickerEnum.eth:
            return tickerEnum.eth;
        case tickerEnum.erc20_usdt:
        case tickerEnum.erc20_crv:
        case tickerEnum.erc20_gtech:
            return tickerEnum.eth;
        default:
            return null;
    };
}

export function isErc20Token(currencyName) {
    switch (currencyName.toUpperCase()) {
        case tickerEnum.btc:
        case tickerEnum.eth:
            return false;
        case tickerEnum.erc20_usdt:
        case tickerEnum.erc20_crv:
        case tickerEnum.erc20_gtech:
            return true;
        default:
            return null;
    };
}
