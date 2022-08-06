const fileSelector = document.getElementById("keyfile-input");
const fileSelectorStatus = document.getElementById("keyfile-input-status");
const currencyWrapper = document.getElementById("currency-wrapper");
const hotWalletBalanceWrapper = document.getElementById("hot-wallet-balance-wrapper");
const withdrawalRequestsWrapper = document.getElementById("withdrawal-requests-wrapper");

let privateKeyJwk;
let publicKeyDer;

async function loadTemplate(path) {
    const template = await (await fetch(path)).text();
    return Handlebars.compile(template);
}

function registerHelpers() {
    Handlebars.registerHelper('currencyName', (currency) => {
        return (typeof currency === 'object' ? currency.ERC20.ticker : currency);
    });
    Handlebars.registerHelper('truncate', (text, n) => {
        return text.substring(0, n) + "...";
    });
    Handlebars.registerHelper('truncateMiddle', (text, n) => {
        if (text.length > n) {
            let left = Math.ceil(n / 2);
            let right = Math.floor(n / 2);
            return text.substring(0, left) + '...' + text.substring(text.length - right, text.length);
        }
        return text;
    });
    Handlebars.registerHelper('formatAddress', addressToString);
    Handlebars.registerHelper('formatStatus', statusToString);
    Handlebars.registerHelper('isCompleted', (status) => { return status.type === "Completed" });
    Handlebars.registerHelper('formatExplorerLink', (txid) => {
        switch (txid.type) {
            case "BTC":
                return "https://mempool.space/tx/" + txid.txid;
            case "ETH":
                return "https://etherscan.io/tx/" + txid.txid;
            default:
                return "unknown";
        };
    })
}

function addressToString(address) {
    switch (address.type) {
        case "BTC":
            return address.addr;
        case "ETH":
            return address.account;
        default:
            return "unknown";
    };
}

function statusToString(status, requiredConfirmations) {
    switch (status.type) {
        case "InProgress":
            return "In progress (" + status.confirmations + " of " + requiredConfirmations + ")";
        case "Confirmed":
            return "Confirmed";
        case "OpRejected":
            return "Rejected by operators";
        case "NodeRejected":
            return "Rejected by node (" + status.reason + ")";
        case "Completed":
            return "Completed";
        default:
            return "Unknown";
    };
}

async function importKey(_event) {
    const keyObj = new window.jscu.Key('pem', this.result);
    if (keyObj.isEncrypted) {
        const password = window.prompt("Enter password");
        try {
            await keyObj.decrypt(password);
        } catch (_error) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = "Wrong password";
            clearData();
            return;
        };
    };
    if (!keyObj.isPrivate) {
        fileSelectorStatus.className = "text-error";
        fileSelectorStatus.innerText = "The selected key is not private";
        clearData();
        return;
    } else {
        try {
            privateKeyJwk = await keyObj.export('jwk');
            publicKeyDer = await keyObj.export('der', { outputPublic: true, compact: true });
        } catch (error) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = error;
            clearData();
            return;
        };
    };
    const response = await getSupportedCurrencies();
    if (response.ok) {
        fileSelectorStatus.className = "text-success";
        fileSelectorStatus.innerText = "Private key imported successfully";
        const currencies = await response.json();
        await loadData(currencies);
    } else {
        if (response.status == 403) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = "Invalid key";
            clearData();
            return;
        } else {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = response.text();
            clearData();
        }
    }
}

async function loadKeyFile(event) {
    fileSelectorStatus.className = "";
    fileSelectorStatus.innerText = "Loading...";
    const file = event.target.files.item(0);
    const reader = new FileReader();
    reader.readAsText(file);
    reader.onload = importKey;
    reader.onerror = () => {
        fileSelectorStatus.className = "text-error";
        fileSelectorStatus.innerText = reader.error;
    };
}

async function makeSignedRequest(requestBody, url, method) {
    const full_url = window.location.href + url;
    const nonce = Date.now();
    const msg_elements = requestBody ? [full_url, JSON.stringify(requestBody), nonce] : [full_url, nonce];
    const msg = msg_elements.join(':');
    const encoder = new TextEncoder();
    const binaryMsg = encoder.encode(msg);
    const signature = await window.jscec.sign(binaryMsg, privateKeyJwk, 'SHA-256', 'der').catch(error => {
        alert(error);
    });
    const signature_data_elements = [
        Base64.fromUint8Array(signature),
        nonce.toString(),
        Base64.fromUint8Array(publicKeyDer)
    ];
    const signature_data = signature_data_elements.join(':');
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
        };
    const response = await fetch(url, params);
    return response;
}

async function getSupportedCurrencies() {
    const response = await makeSignedRequest(null, "currencies", 'GET');
    return response;
}

async function getRequiredConfirmations() {
    const response = await makeSignedRequest(null, "confirmations", 'GET');
    if (response.ok) {
        let data = await response.json();
        return data;
    } else {
        alert("Error: " + response.text);
        return;
    }
}

async function loadHotWalletBalance(currency) {
    hotBalanceTemplate = await loadTemplate("/templates/hot_balance.html.hbs");
    const response = await makeSignedRequest(null, "hotbalance", "GET");
    if (response.ok) {
        let data = await response.json();
        let value = data.balance / 100000000;
        const context = { balance: value, units: currency };
        const hotBalanceHTML = hotBalanceTemplate(context);
        hotWalletBalanceWrapper.innerHTML = hotBalanceHTML;
    } else {
        alert("Error: " + response.text);
    }
}

// Adds tooltips, enables copy buttons
function prettifyWithdrawalRequestsTable(withdrawalRequests) {
    const withdrawalRequestElements = document.getElementsByClassName("withdrawal-request-row");
    let withdrawalRequestElement;
    let withdrawalRequestIdElement;
    for (let i = 0; i < withdrawalRequestElements.length; i++) {
        withdrawalRequestElement = withdrawalRequestElements[i];

        // Add tooltips and enable copy button to withdrawal request ID
        withdrawalRequestIdElement = withdrawalRequestElement.getElementsByClassName("id-cell")[0];
        withdrawalRequestIdTextElement = withdrawalRequestIdElement.getElementsByTagName("span")[0];
        tippy(withdrawalRequestIdTextElement, {
            content: withdrawalRequests[i].id,
        });
        withdrawalRequestIdCopyBtnElement = withdrawalRequestIdElement.getElementsByTagName("button")[0];
        withdrawalRequestIdCopyBtnElement.addEventListener("click", () => {
            navigator.clipboard.writeText(withdrawalRequests[i].id).then(function () { }, function (err) {
                console.error('Could not copy text: ', err);
            });
        });
        tippy(withdrawalRequestIdCopyBtnElement, {
            content: "Copied",
            trigger: "click",
            hideOnClick: false,
            onShow(instance) {
                setTimeout(() => {
                    instance.hide();
                }, 1000);
            }
        });

        // Add tooltips and enable copy button to address
        addressElement = withdrawalRequestElement.getElementsByClassName("address-cell")[0];
        addressTextElement = addressElement.getElementsByTagName("span")[0];
        tippy(addressTextElement, {
            content: addressToString(withdrawalRequests[i].address),
        });

        addressCopyBtnElement = addressElement.getElementsByTagName("button")[0];
        addressCopyBtnElement.addEventListener("click", () => {
            navigator.clipboard.writeText(addressToString(withdrawalRequests[i].address)).then(function () { }, function (err) {
                console.error('Could not copy text: ', err);
            });
        });
        tippy(addressCopyBtnElement, {
            content: "Copied",
            trigger: "click",
            hideOnClick: false,
            onShow(instance) {
                setTimeout(() => {
                    instance.hide();
                }, 1000);
            }
        });

        // Add tooltip to explorer link if it exists
        statusElement = withdrawalRequestElement.getElementsByClassName("status-cell")[0];
        if (statusElement.getElementsByTagName("a").lengh > 0) {
            statusTxIdBtnElement = statusElement.getElementsByTagName("a")[0];
            tippy(statusTxIdBtnElement, {
                content: "Link to the explorer",
            });
        }
    }
}

async function loadWithdrawalRequests(currency) {
    withdrawalRequestsTemplate = await loadTemplate("/templates/withdrawal_requests.html.hbs");
    const response = await makeSignedRequest(null, "request", 'GET');
    if (response.ok) {
        const withdrawalRequests = await response.json();
        const sortedWithdrawalRequests = withdrawalRequests.sort((a, b) => {
            const dateA = new Date(a.created_at);
            const dateB = new Date(b.created_at);
            return dateA - dateB;
        });
        const totatlRequiredConfirmations = await getRequiredConfirmations();
        const context = { withdrawalRequests: sortedWithdrawalRequests, requiredConfirmations: totatlRequiredConfirmations };
        const withdrawalRequestsHTML = withdrawalRequestsTemplate(context);
        withdrawalRequestsWrapper.innerHTML = withdrawalRequestsHTML;
        prettifyWithdrawalRequestsTable(sortedWithdrawalRequests);
    } else {
        alert("Error: " + response.text);
    }
}

async function loadData(supportedCurrencies) {
    currencySelectTemplate = await loadTemplate("/templates/currency_select.html.hbs");
    const context = { currencies: supportedCurrencies };
    const currencySelectHTML = currencySelectTemplate(context);
    currencyWrapper.innerHTML = currencySelectHTML;
    const currencySelect = document.getElementById("currency-select");
    currencySelect.addEventListener("change", () => {
        let selectedCurrency = currencySelect.options[currencySelect.selectedIndex].text;
        loadHotWalletBalance(selectedCurrency);
        loadWithdrawalRequests(selectedCurrency);
    });
    let selectedCurrency = currencySelect.options[currencySelect.selectedIndex].text;
    loadHotWalletBalance(selectedCurrency);
    loadWithdrawalRequests(selectedCurrency);
}

function clearData() {
    currencyWrapper.innerHTML = "";
    withdrawalRequestsWrapper.innerHTML = "";
    hotWalletBalanceWrapper.innerHTML = "";
}

async function confirmRequest(confirmationData) {
    const response = await makeSignedRequest(confirmationData, 'confirm', 'POST');
    if (response.ok) {
        await loadWithdrawalRequests();
    } else {
        alert("Error: " + response.text);
    };
}

async function rejectRequest(confirmationData) {
    const response = await makeSignedRequest(confirmationData, 'reject', 'POST');
    if (response.ok) {
        await loadWithdrawalRequests();
    } else {
        alert("Error: " + response.text);
    };
}

function addActionBtns(row, withdrawalRequest, requestStatus) {
    // Here we copy withdrawal_request and remove confirmation status feild
    let confirmationData = (({ confirmation_status, ...o }) => o)(withdrawalRequest);

    let disabled = (requestStatus == "InProgress") ? false : true;
    let cell = document.createElement("td");
    let btnRow = document.createElement("div");
    btnRow.setAttribute("class", "row pull-left");

    let confirmBtnCol = document.createElement("div");
    confirmBtnCol.setAttribute("class", "col");
    let confirmBtn = document.createElement("button");
    confirmBtn.addEventListener("click", () => { confirmRequest(confirmationData); });
    let confirmBtnText = document.createTextNode("Confirm")
    confirmBtn.appendChild(confirmBtnText);
    confirmBtn.setAttribute("class", "button primary");
    if (disabled) {
        confirmBtn.setAttribute("disabled", "true");
    };
    confirmBtnCol.appendChild(confirmBtn);
    btnRow.appendChild(confirmBtnCol);

    let rejectBtnCol = document.createElement("div");
    rejectBtnCol.setAttribute("class", "col");
    let rejectBtn = document.createElement("button");
    rejectBtn.addEventListener("click", () => { rejectRequest(confirmationData); });
    let rejectBtnText = document.createTextNode("Reject")
    rejectBtn.appendChild(rejectBtnText);
    rejectBtn.setAttribute("class", "button error");
    if (disabled) {
        rejectBtn.setAttribute("disabled", "true");
    };
    rejectBtnCol.appendChild(rejectBtn);
    btnRow.appendChild(rejectBtnCol);

    cell.appendChild(btnRow);
    row.appendChild(cell);
}

window.addEventListener('load', function () {
    registerHelpers();
    fileSelector.addEventListener('change', loadKeyFile);
});
