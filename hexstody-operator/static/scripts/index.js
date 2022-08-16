import { loadTemplate, formattedCurrencyValue, initTabs, openTab } from "./common.js";

const fileSelector = document.getElementById("keyfile-input");
const fileSelectorStatus = document.getElementById("keyfile-input-status");
const authorizedInfoWrapper = document.getElementById("authorized-content-wrapper");

let privateKeyJwk;
let publicKeyDer;

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
    });
    Handlebars.registerHelper('truncateChangeId', function () { return truncate(this.id, 10) });
    Handlebars.registerHelper('limitsFormatName', function () { return getCurName(this.currency) });
    Handlebars.registerHelper('renderChangeStatus', function () { return renderChangeStatus(this.status) });
    Handlebars.registerHelper('renderCurLimit', function () { return renderLimit(this.current_limit) });
    Handlebars.registerHelper('renderNewLimit', function () { return renderLimit(this.requested_limit) });
    Handlebars.registerHelper('renderTime', function () {
        const d = new Date(this.created_at);
        if (d instanceof Date && !isNaN(d)) {
            return d.getDate() + "." + d.getMonth() + "." + d.getFullYear() + " " + d.toLocaleTimeString()
        } else { return "Invalid time" }
    });
}


function renderLimit(limit) {
    return limit.amount + "/" + limit.span
}

function renderChangeStatus(status) {
    switch (Object.keys(status)[0]) {
        case "InProgress":
            let body = status["InProgress"];
            return "In progress (+" + body.confirmations + "/-" + body.rejections + " of 2)";
        case "Confirmed":
            return "Confirmed";
        case "Rejected":
            return "Rejected by operators";
        default:
            cellText = document.createTextNode("Unknown");
    };
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
            clearAuthorizedWrapper();
            return;
        };
    };
    if (!keyObj.isPrivate) {
        fileSelectorStatus.className = "text-error";
        fileSelectorStatus.innerText = "The selected key is not private";
        clearAuthorizedWrapper();
        return;
    } else {
        try {
            privateKeyJwk = await keyObj.export('jwk');
            publicKeyDer = await keyObj.export('der', { outputPublic: true, compact: true });
        } catch (error) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = error;
            clearAuthorizedWrapper();
            return;
        };
    };
    const response = await getSupportedCurrencies();
    if (response.ok) {
        fileSelectorStatus.className = "text-success";
        fileSelectorStatus.innerText = "Private key imported successfully";
        // Here we test the key and get the list of supported currencies
        const currencies = await response.json();
        await loadAuthorizedContent(currencies);
    } else {
        if (response.status == 403) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = "Invalid key";
            clearAuthorizedWrapper();
            return;
        } else {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = response.text();
            clearAuthorizedWrapper();
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

async function getAllChanges() {
    return await makeSignedRequest(null, "changes", "GET").then(r => r.json())
}

async function loadHotWalletBalance(currency) {
    const hotWalletBalanceWrapper = document.getElementById("hot-wallet-balance-wrapper");
    const hotBalanceTemplate = await loadTemplate("/templates/hot_balance.html.hbs");
    const response = await makeSignedRequest(null, `hot-wallet-balance/${currency.toLowerCase()}`, "GET");
    if (response.ok) {
        let data = await response.json();
        let value = formattedCurrencyValue(currency, data.balance);
        const context = { balance: value, units: currency };
        const hotBalanceHTML = hotBalanceTemplate(context);
        hotWalletBalanceWrapper.innerHTML = hotBalanceHTML;
    } else {
        alert("Error: " + response.text);
    }
}

async function confirmRequest(confirmationData, currency) {
    const response = await makeSignedRequest(confirmationData, 'confirm', 'POST');
    if (response.ok) {
        await loadWithdrawalRequests(currency);
    } else {
        alert("Error: " + response.text);
    };
}

async function rejectRequest(confirmationData, currency) {
    const response = await makeSignedRequest(confirmationData, 'reject', 'POST');
    if (response.ok) {
        await loadWithdrawalRequests(currency);
    } else {
        alert("Error: " + response.text);
    };
}

function handleConfirm(change) {
    return async function () {
        const body = {
            id: change.id,
            user: change.user,
            currency: change.currency,
            created_at: change.created_at,
            requested_limit: change.requested_limit,
        };
        await makeSignedRequest(body, "limits/confirm", "POST");
        loadWithdrawalLimitsTab()
    }
}

function handleReject(change) {
    return async function () {
        const body = {
            id: change.id,
            user: change.user,
            currency: change.currency,
            created_at: change.created_at,
            requested_limit: change.requested_limit,
        };
        await makeSignedRequest(body, "limits/reject", "POST");
        loadWithdrawalLimitsTab()
    }
}

// Adds tooltips, enables copy buttons
function prettifyWithdrawalRequestsTable(withdrawalRequests, currency) {
    const withdrawalRequestElements = document.getElementsByClassName("withdrawal-request-row");
    for (let i = 0; i < withdrawalRequestElements.length; i++) {
        let withdrawalRequestElement = withdrawalRequestElements[i];

        // Add tooltips and enable copy button to withdrawal request ID
        let withdrawalRequestIdElement = withdrawalRequestElement.getElementsByClassName("id-cell")[0];
        let withdrawalRequestIdTextElement = withdrawalRequestIdElement.getElementsByTagName("span")[0];
        tippy(withdrawalRequestIdTextElement, {
            content: withdrawalRequests[i].id,
        });
        let withdrawalRequestIdCopyBtnElement = withdrawalRequestIdElement.getElementsByTagName("button")[0];
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
        let addressElement = withdrawalRequestElement.getElementsByClassName("address-cell")[0];
        let addressTextElement = addressElement.getElementsByTagName("span")[0];
        tippy(addressTextElement, {
            content: addressToString(withdrawalRequests[i].address),
        });

        let addressCopyBtnElement = addressElement.getElementsByTagName("button")[0];
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
        let statusElement = withdrawalRequestElement.getElementsByClassName("status-cell")[0];
        if (statusElement.getElementsByTagName("a").lengh > 0) {
            statusTxIdBtnElement = statusElement.getElementsByTagName("a")[0];
            tippy(statusTxIdBtnElement, {
                content: "Link to the explorer",
            });
        }

        // Add "Confirm" and "Reject" buttons
        let actionButtonsElement = withdrawalRequestElement.getElementsByClassName("action-buttons-cell")[0];
        let isDisabled = (withdrawalRequests[i].confirmation_status.type == "InProgress") ? false : true;
        // Here we copy withdrawal_request and remove confirmation status feild
        let confirmationData = (({ confirmation_status, ...o }) => o)(withdrawalRequests[i]);
        let confirmBtn = actionButtonsElement.getElementsByTagName("button")[0];
        confirmBtn.addEventListener("click", () => { confirmRequest(confirmationData, currency); });
        if (isDisabled) {
            confirmBtn.setAttribute("disabled", "true");
        };
        let rejectBtn = actionButtonsElement.getElementsByTagName("button")[1];
        rejectBtn.addEventListener("click", () => { rejectRequest(confirmationData, currency); });
        if (isDisabled) {
            rejectBtn.setAttribute("disabled", "true");
        };
    }
}

async function loadWithdrawalRequestsTab(supportedCurrencies) {
    openTab("navigation-tabs", "withdrawal-requests-tab");

    const withdrawalRequestsTabContent = document.getElementById("withdrawal-requests-tab-content");
    withdrawalRequestsTabContent.innerHTML = "";

    const currencySelectWrapper = document.createElement("div");
    currencySelectWrapper.id = "currency-select-wrapper";
    currencySelectWrapper.classList.add('mb-1');
    withdrawalRequestsTabContent.appendChild(currencySelectWrapper);

    const hotWalletBalanceWrapper = document.createElement("div");
    hotWalletBalanceWrapper.id = "hot-wallet-balance-wrapper";
    hotWalletBalanceWrapper.classList.add('mb-1');
    withdrawalRequestsTabContent.appendChild(hotWalletBalanceWrapper);

    const withdrawalRequestsWrapper = document.createElement("div");
    withdrawalRequestsWrapper.id = "withdrawal-requests-wrapper";
    withdrawalRequestsTabContent.appendChild(withdrawalRequestsWrapper);

    const currencySelectTemplate = await loadTemplate("/templates/currency_select.html.hbs");
    const context = { currencies: supportedCurrencies };
    const currencySelectHTML = currencySelectTemplate(context);
    currencySelectWrapper.innerHTML = currencySelectHTML;

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

async function loadWithdrawalRequests(currency) {
    const withdrawalRequestsWrapper = document.getElementById("withdrawal-requests-wrapper");
    const withdrawalRequestsTemplate = await loadTemplate("/templates/withdrawal_requests.html.hbs");
    const response = await makeSignedRequest(null, `request/${currency.toLowerCase()}`, 'GET');
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
        prettifyWithdrawalRequestsTable(sortedWithdrawalRequests, currency);
    } else {
        alert("Error: " + response.text);
    }
}

async function loadAuthorizedContent(supportedCurrencies) {
    clearAuthorizedWrapper();
    const navigationTabsTemplate = await loadTemplate("/templates/navigation_tabs.html.hbs");
    const navigationTabsHTML = navigationTabsTemplate();
    authorizedInfoWrapper.insertAdjacentHTML('beforeend', navigationTabsHTML);
    initTabs("navigation-tabs");
    loadWithdrawalRequestsTab(supportedCurrencies);
    const withdrawalRequestsTab = document.getElementById("withdrawal-requests-tab");
    withdrawalRequestsTab.addEventListener("click", () => { loadWithdrawalRequestsTab(supportedCurrencies); });
    const invitesTab = document.getElementById("invites-tab");
    invitesTab.addEventListener("click", loadInvitesTab);
    const withdrawalLimitsTab = document.getElementById("withdrawal-limits-tab");
    withdrawalLimitsTab.addEventListener("click", loadWithdrawalLimitsTab);
}

function clearAuthorizedWrapper() {
    authorizedInfoWrapper.innerHTML = "";
}

async function getInvite(body) {
    return await makeSignedRequest(body, "invite/generate", "POST");
};

async function getMyInvites() {
    return await makeSignedRequest(null, "invite/listmy", "GET");
}

function mkCopyBtn(parent, value) {
    const copyBtn = document.createElement("button");
    copyBtn.classList.add("button", "clear", "icon-only");
    copyBtn.innerHTML = `<img src="/images/copy.svg?size=18" class="icon-18px"></img>`;
    parent.appendChild(copyBtn);
    copyBtn.addEventListener("click", () => {
        navigator.clipboard.writeText(value).then(function () { }, function (err) {
            console.error('Could not copy text: ', err);
        });
    });
    tippy(copyBtn, {
        content: "Copied",
        trigger: "click",
        hideOnClick: false,
        onShow(instance) {
            setTimeout(() => {
                instance.hide();
            }, 1000);
        }
    });
}

function clearError(errorContainer) {
    errorContainer.innerHTML = "";
}

function renderError(errorContainer, errMsg) {
    errorContainer.innerHTML = `<p>${errMsg}</p>`;
}

async function genInvite() {
    const inviteLabelField = document.getElementById("invite-label");
    const inviteDisplay = document.getElementById("invite-display");
    const inviteError = document.getElementById("invite-error");
    const label = inviteLabelField.value;
    if (!label) {
        renderError(inviteError, "Label field is required");
        inviteDisplay.style.display = 'none';
    } else {
        clearError(inviteError);
        const body = { label: label }
        const inviteResp = await getInvite(body)
        if (inviteResp.ok) {
            let invite = await inviteResp.json();
            const invitesListTemplate = await loadTemplate("/templates/inviteslist.html.hbs");
            const invitesHTML = invitesListTemplate({ invites: [invite], isList: false });
            inviteDisplay.style.display = 'block';
            inviteDisplay.innerHTML = invitesHTML;
            const copyBtnParent = inviteDisplay.querySelector(".invite-display");
            mkCopyBtn(copyBtnParent, invite.invite.invite);
        } else {
            renderError(inviteError, `Failed do generate an invite: ${JSON.stringify(inviteResp)}`);
        }
    }
}

async function listInvites() {
    const invitesListBody = document.getElementById("invites-list");
    const invitesResp = await getMyInvites();
    if (invitesResp.ok) {
        const invites = await invitesResp.json();
        const invitesListTemplate = await loadTemplate("/templates/inviteslist.html.hbs");
        const invitesList = invitesListTemplate({ invites: invites, isList: true });
        invitesListBody.innerHTML = invitesList;
        const inviteDisplays = invitesListBody.querySelectorAll(".invite-display");
        inviteDisplays.forEach(display => {
            const value = display.querySelector('.invite').innerHTML;
            mkCopyBtn(display, value.trim())
        })
    } else {
        const errMsg = document.createElement("p");
        errMsg.classList.add("text-error");
        errMsg.innerHTML = "Error: " + invitesResp.status + ": " + invitesResp.statusText;
        invitesListBody.innerHTML = "";
        invitesListBody.append(errMsg)
    }
}

async function loadInvitesTab() {
    openTab("navigation-tabs", "invites-tab");
    const invitesTabContent = document.getElementById("invites-tab-content");
    const inivitesTemplate = await loadTemplate("/templates/invites.html.hbs");
    invitesTabContent.innerHTML = inivitesTemplate();
    const genInviteBtn = document.getElementById("gen-invite-btn");
    const listInvitesBtn = document.getElementById("btn-list-invites");
    genInviteBtn.onclick = genInvite;
    listInvitesBtn.onclick = listInvites;
}

async function loadWithdrawalLimitsTab() {
    console.log("hello");
    openTab("navigation-tabs", "withdrawal-limits-tab");
    const withdrawalLimitsTabContent = document.getElementById("withdrawal-limits-tab-content");
    const withdrawalLimitsTemplate = await loadTemplate("/templates/withdrawal_limits.html.hbs");
    const changes = await getAllChanges();
    withdrawalLimitsTabContent.innerHTML = withdrawalLimitsTemplate({ changes: changes });
    changes.forEach(change => {
        const confirmBtn = document.getElementById(change.id + "-confirm");
        const rejectBtn = document.getElementById(change.id + "-reject");
        const idSpan = document.getElementById(change.id + "-id");
        tippy(idSpan, { content: change.id });
        confirmBtn.onclick = handleConfirm(change);
        rejectBtn.onclick = handleReject(change);
    })
}

window.addEventListener('load', function () {
    registerHelpers();
    fileSelector.addEventListener('change', loadKeyFile);
});
