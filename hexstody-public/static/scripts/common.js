const SECOND = 1000
const MINUTE = 60 * SECOND
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR

// Amount of satoshi in 1 BTC
const BTC_PRECISION = 10 ** 8
// Amount of wei in 1 ETH
const ETH_PRECISION = 10 ** 18
const USDT_PRECISION = 10 ** 6
const CRV_PRECISION = 10 ** 18
const GTECH_PRECISION = 10 ** 18

export const GWEI = 10 ** 9

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
            "contract": "0xdAC17F958D2ee523a2206206994597C13D831ec7"
        }
    },
    erc20_crv: {
        "ERC20": {
            "ticker": tickerEnum.erc20_crv,
            "name": "CRV",
            "contract": "0xd533a949740bb3306d119cc777fa900ba034cd52"
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

export function* getAllCurrencies() {
    for (const currencyKey in currencyEnum) {
        const currency = currencyEnum[currencyKey]
        if (typeof currency === "string") yield currency
        if (typeof currency === "object" && currency.ERC20 !== undefined) yield currency.ERC20.ticker
    }
}

// Gas limit for ETH transfer transaction
export const ETH_TX_GAS_LIMIT = 21_000
// Gas limit for ERC20 transfer transaction
export const ERC20_TX_GAS_LIMIT = 150_000

export async function loadTemplate(path) {
    const template = await (await fetch(path)).text()
    return Handlebars.compile(template)
}

export function formattedCurrencyValue(currency, value) {
    let numberFormat
    switch (currency) {
        case "BTC":
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(BTC_PRECISION),
            })
            return numberFormat.format(value / BTC_PRECISION)
        case "ETH":
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(ETH_PRECISION),
            })
            return numberFormat.format(value / ETH_PRECISION)
        case "USDT":
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(USDT_PRECISION),
            })
            return numberFormat.format(value / USDT_PRECISION)
        case "CRV":
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(CRV_PRECISION),
            })
            return numberFormat.format(value / CRV_PRECISION)
        case "GTECH":
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(GTECH_PRECISION),
            })
            return numberFormat.format(value / GTECH_PRECISION)
        default:
            return value
    };
}

export function formattedElapsedTime(dateTimeString) {
    const date = new Date(dateTimeString)
    const currentDate = new Date()
    const localOffset = currentDate.getTimezoneOffset() * MINUTE
    const msElapsed = currentDate - date + localOffset
    const rtf = new Intl.RelativeTimeFormat('en', {
        numeric: 'auto'
    })
    function fmt(constant, constantString) {
        return rtf.format(-Math.round(msElapsed / constant), constantString)
    }

    if (msElapsed < MINUTE) {
        return fmt(SECOND, "second")
    } else if (msElapsed < HOUR) {
        return fmt(MINUTE, "minute")
    } else if (msElapsed < DAY) {
        return fmt(HOUR, "hour")
    } else if (msElapsed < DAY * 2) {
        return fmt(DAY, "day")
    } else {
        const localDate = date.getTime() - localOffset
        return new Date(localDate).toLocaleString()
    }
}

export function initTabs(tabIds, hook, selected) {
    function tabClicked(clickedTabId) {
        tabIds.forEach(tabId => {
            const validationDisplay = document.getElementById(tabId + "-body")
            if (tabId === clickedTabId) {
                document.getElementById(tabId).classList.add("is-active")
                validationDisplay.style.display = "block"
            } else {
                document.getElementById(tabId).classList.remove("is-active")
                validationDisplay.style.display = "none"
            }
        })
        if (typeof hook === 'function') {
            hook(clickedTabId)
        }
    }
    tabIds.forEach(tab => document.getElementById(tab).onclick = () => tabClicked(tab))
    var i
    if (selected) { i = selected } else { i = 0 };
    tabClicked(tabIds[i])
}

export function initCollapsibles() {
    console.log("A")
    const cols = document.getElementsByClassName("collapsible")
    console.log(cols)
    for (let col of cols) {
        col.addEventListener("click", function () {
            this.classList.toggle("active")
            const content = this.nextElementSibling
            if (content.style.display === "block") {
                content.style.display = "none"
            } else {
                content.style.display = "block"
            }
        })
    }
    // .forEach(function (coll) {

    // });
}

export function initDropDowns() {
    var $dropdowns = getAll('.dropdown:not(.is-hoverable)')

    if ($dropdowns.length > 0) {
        $dropdowns.forEach(function ($el) {
            $el.addEventListener('click', function (event) {
                event.stopPropagation()
                $el.classList.toggle('is-active')
            })
        })

        document.addEventListener('click', function (event) {
            closeDropdowns()
        })
    }

    function closeDropdowns() {
        $dropdowns.forEach(function ($el) {
            $el.classList.remove('is-active')
        })
    }

    // Close dropdowns if ESC pressed
    document.addEventListener('keydown', function (event) {
        var e = event || window.event
        if (e.key === "Escape") {
            closeDropdowns()
        }
    })

    // Functions
    function getAll(selector) {
        return Array.prototype.slice.call(document.querySelectorAll(selector), 0)
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
        const chunk = array.slice(i, i + chunkSize)
        chunks.push(chunk)
    }
    return chunks
}

export function transpose(array) {
    var transposed = []
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
        const chunk = array.slice(i, i + chunkSize)
        for (let j = 0; j < chunkSize; j++) {
            res[j].push(chunk[j])
        }
    }

    return res
}

export function indexArrayFromOne(array) {
    var res = []
    for (let i = 0; i < array.length; i++) {
        res.push({ ix: i + 1, value: array[i] })
    }
    return res
}

export function currencyNameToCurrency(currencyName) {
    switch (currencyName.toUpperCase()) {
        case "BTC":
            return currencyEnum.btc
        case "ETH":
            return currencyEnum.eth
        case "USDT":
            return currencyEnum.erc20_usdt
        case "CRV":
            return currencyEnum.erc20_crv
        case "GTECH":
            return currencyEnum.erc20_gtech
        default:
            return null
    }
}

export function currencyPrecision(currencyName) {
    switch (currencyName.toUpperCase()) {
        case "BTC":
            return BTC_PRECISION
        case "ETH":
            return ETH_PRECISION
        case "USDT":
            return USDT_PRECISION
        case "CRV":
            return CRV_PRECISION
        case "GTECH":
            return GTECH_PRECISION
        default:
            return null
    };
}

// Converts amounts in whole units to smallest units.
// E.g. ETH to WEI, BTC to sats and so on.
export function convertToSmallest(currency, value) {
    return value * currencyPrecision(currency)
}

// The currency in which transaction fees are paid
export function feeCurrency(currencyName) {
    switch (currencyName.toUpperCase()) {
        case "BTC":
            return "BTC"
        case "ETH":
            return "ETH"
        case "USDT":
        case "CRV":
        case "GTECH":
            return "ETH"
        default:
            return null
    };
}

export function isErc20Token(currencyName) {
    switch (currencyName.toUpperCase()) {
        case "BTC":
        case "ETH":
            return false
        case "USDT":
        case "CRV":
        case "GTECH":
            return true
        default:
            return null
    };
}
