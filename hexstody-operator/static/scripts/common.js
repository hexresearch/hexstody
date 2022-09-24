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

export const currencyEnum = Object.freeze({
    btc: "BTC",
    eth: "ETH",
    erc20_usdt: {
        "ERC20": {
            "ticker": "USDT",
            "name": "USDT",
            "contract": "0xfD8ef4113c5f54BE9Cb103eB437b710b8e1d6885"
        }
    },
    erc20_crv: {
        "ERC20": {
            "ticker": "CRV",
            "name": "CRV",
            "contract": "0x817805F0f818237c73Fde5dEc91dbB650A7E7612"
        }
    },
    erc20_gtech: {
        "ERC20": {
            "ticker": "GTECH",
            "name": "GTECH",
            "contract": "0x866A4Da32007BA71aA6CcE9FD85454fCF48B140c"
        }
    }
})

export function isErc20Token(currency) {
    if (currency !== null && typeof currency === 'object') {
        if ('ERC20' in currency) {
            return true
        } else {
            return false
        }
    } else {
        return false
    };
}

export function getCurrencyName(currency) {
    if (isErc20Token(currency)) {
        return currency.ERC20.name
    } else {
        return currency
    };
}

export function formatCurrencyValue(currency, value) {
    let numberFormat
    switch (currency) {
        case currencyEnum.btc:
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(BTC_PRECISION),
            })
            return numberFormat.format(value / BTC_PRECISION)
        case currencyEnum.eth:
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(ETH_PRECISION),
            })
            return numberFormat.format(value / ETH_PRECISION)
        case currencyEnum.erc20_usdt:
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(USDT_PRECISION),
            })
            return numberFormat.format(value / USDT_PRECISION)
        case currencyEnum.erc20_crv:
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(CRV_PRECISION),
            })
            return numberFormat.format(value / CRV_PRECISION)
        case currencyEnum.erc20_gtech:
            numberFormat = Intl.NumberFormat('en', {
                maximumFractionDigits: Math.log10(GTECH_PRECISION),
            })
            return numberFormat.format(value / GTECH_PRECISION)
        default:
            return value
    };
}

export function truncate(text, n) {
    return text.substring(0, n) + "..."
}

export function truncateMiddle(text, n) {
    if (text.length > n) {
        let left = Math.ceil(n / 2)
        let right = Math.floor(n / 2)
        return text.substring(0, left) + '...' + text.substring(text.length - right, text.length)
    }
    return text
}

export function formatAddress(address) {
    switch (address.type) {
        case "BTC":
            return address.addr
        case "ETH":
            return address.account
        default:
            return "unknown"
    };
}

export function formatWithdrawalRequestStatus(status, requiredConfirmations) {
    switch (status.type) {
        case "InProgress":
            return "In progress (" + status.confirmations + " of " + requiredConfirmations + ")"
        case "Confirmed":
            return "Confirmed"
        case "OpRejected":
            return "Rejected by operators"
        case "NodeRejected":
            return "Rejected by node"
        case "Completed":
            return "Completed"
        default:
            return "Unknown"
    };
}

export function formatExplorerLink(txid) {
    switch (txid.type) {
        case "BTC":
            return "https://mempool.space/tx/" + txid.txid
        case "ETH":
            return "https://etherscan.io/tx/" + txid.txid
        default:
            return "unknown"
    };
}

export function formatLimitTime(datetime) {
    const time = new Date(datetime)
    const dateStr = `${time.getFullYear()}-${String(time.getMonth() + 1).padStart(2, '0')}-${String(time.getDate()).padStart(2, '0')}`
    const timeStr = time.toLocaleTimeString()
    if (time instanceof Date && !isNaN(time)) {
        return `${dateStr} ${timeStr}`
    } else {
        return "Invalid time"
    }
}

export function formatLimitValue(limit) {
    return limit.amount + " / " + limit.span
}

export function formatLimitStatus(status) {
    switch (status.type) {
        case "InProgress":
            return "In progress (+" + status.confirmations + " / -" + status.rejections + " of 2)"
        case "Confirmed":
            return "Confirmed"
        case "Rejected":
            return "Rejected by operators"
        default:
            return "Unknown"
    };
}

export function copyToClipboard(text) {
    navigator.clipboard.writeText(text)
}

export async function makeSignedRequest(privateKeyJwk, publicKeyDer, requestBody, url, method) {
    const full_url = window.location.href + url
    const nonce = Date.now()
    const msg_elements = requestBody ? [full_url, JSON.stringify(requestBody), nonce] : [full_url, nonce]
    const msg = msg_elements.join(':')
    const encoder = new TextEncoder()
    const binaryMsg = encoder.encode(msg)
    const signature = await window.jscec.sign(binaryMsg, privateKeyJwk, 'SHA-256', 'der').catch(error => {
        alert(error)
    })
    const signature_data_elements = [
        Base64.fromUint8Array(signature),
        nonce.toString(),
        Base64.fromUint8Array(publicKeyDer)
    ]
    const signature_data = signature_data_elements.join(':')
    const params = requestBody ?
        {
            method: method,
            body: JSON.stringify(requestBody),
            headers: {
                'Content-Type': 'application/json',
                'Signature-Data': signature_data
            }
        } : {
            method: method,
            headers: {
                'Signature-Data': signature_data
            }
        }
    const response = await fetch(url, params)
    return response
}

export async function getSupportedCurrencies(privateKeyJwk, publicKeyDer) {
    const response = await makeSignedRequest(privateKeyJwk, publicKeyDer, null, "currencies", 'GET')
    return response
}

export async function getHotWalletBalance(privateKeyJwk, publicKeyDer, currency) {
    const response = await makeSignedRequest(privateKeyJwk, publicKeyDer, null, `hot-wallet-balance/${getCurrencyName(currency).toLowerCase()}`, 'GET')
    return response
}

export async function getWithdrawalRequests(privateKeyJwk, publicKeyDer, currency, filter) {
    const response = await makeSignedRequest(privateKeyJwk, publicKeyDer, null, `request/${getCurrencyName(currency).toLowerCase()}?filter=`+filter, 'GET')
    return response
}

export async function getLimitRequests(privateKeyJwk, publicKeyDer, filter) {
    const response = await makeSignedRequest(privateKeyJwk, publicKeyDer, null, "changes?filter=" + filter, "GET")
    return response
}

export async function getRequiredConfirmations(privateKeyJwk, publicKeyDer) {
    const response = await makeSignedRequest(privateKeyJwk, publicKeyDer, null, "confirmations", 'GET')
    return response
}

export async function confirmWithdrawalRequest(privateKeyJwk, publicKeyDer, confirmationData) {
    const response = await makeSignedRequest(privateKeyJwk, publicKeyDer, confirmationData, 'confirm', 'POST')
    return response
}

export async function rejectWithdrawalRequest(privateKeyJwk, publicKeyDer, confirmationData) {
    const response = await makeSignedRequest(privateKeyJwk, publicKeyDer, confirmationData, 'reject', 'POST')
    return response
}

export async function confirmLimitRequest(privateKeyJwk, publicKeyDer, confirmationData) {
    const response = await makeSignedRequest(privateKeyJwk, publicKeyDer, confirmationData, "limits/confirm", "POST")
    return response
}

export async function rejectLimitRequest(privateKeyJwk, publicKeyDer, confirmationData) {
    const response = await makeSignedRequest(privateKeyJwk, publicKeyDer, confirmationData, "limits/reject", "POST")
    return response
}

export async function getInvite(privateKeyJwk, publicKeyDer, inviteLabel) {
    return await makeSignedRequest(privateKeyJwk, publicKeyDer, inviteLabel, "invite/generate", "POST")
}

export async function getInvites(privateKeyJwk, publicKeyDer) {
    return await makeSignedRequest(privateKeyJwk, publicKeyDer, null, "invite/listmy", "GET")
}

export async function getUserInfo(privateKeyJwk, publicKeyDer, userId) {
    return await makeSignedRequest(privateKeyJwk, publicKeyDer, null, `user/info/${userId}`, "GET")
}

export async function getTicker(currency){
    return await fetch("/ticker/ticker", {
        method: "POST",
        body: JSON.stringify(currency)
    })
}