import { loadTemplate, formattedElapsedTime } from "./common.js";

const fileSelector = document.getElementById("file-selector");
const fileSelectorStatus = document.getElementById("file-selector-status");
const withdrawalRequestsTableBody = document.getElementById("withdrawal-requests-table-body");
const invitesTab = document.getElementById("invites-tab");
const withdrawsTabBody = document.getElementById('withdraw-tab-body');
const withdrawsTab = document.getElementById('withdraw-tab');
const invitesTabBody = document.getElementById("invites-tab-body");
const authedBody = document.getElementById("authed-body");

let invitesTemplate = null;
let invitesListTemplate = null;
let privateKeyJwk;
let publicKeyDer;

function hide_authed(){
    authedBody.style.display = "none";
}

function show_authed(){
    authedBody.style.display = "block";
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
            clearWithdrawalRequests();
            hide_authed();
            return;
        };
    };
    if (!keyObj.isPrivate) {
        fileSelectorStatus.className = "text-error";
        fileSelectorStatus.innerText = "The selected key is not private";
        clearWithdrawalRequests();
        hide_authed();
        return;
    } else {
        try {
            privateKeyJwk = await keyObj.export('jwk');
            publicKeyDer = await keyObj.export('der', { outputPublic: true, compact: true });
        } catch (error) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = error;
            clearWithdrawalRequests();
            hide_authed();
            return;
        };
    };
    show_authed();
    const response = await updateWithdrawalRequests();
    if (response.ok) {
        fileSelectorStatus.className = "text-success";
        fileSelectorStatus.innerText = "Private key imported successfully!";
        const response = await makeSignedRequest(null, "hotbalance", "POST");
        if (response.ok) {
            let data = await response.json();
            let val = data.balance / 100000000;
            let txt = "Hot balance: " + val.toString() + " BTC";
            fileSelectorStatus.className = "text-dark"
            fileSelectorStatus.innerText = txt;
        }
    } else {
        if (response.status == 403) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = "Invalid key";
            clearWithdrawalRequests();
            hide_authed();
            return;
        } else {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = response.text();
            clearWithdrawalRequests();
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

function clearWithdrawalRequests() {
    withdrawalRequestsTableBody.textContent = '';
}

function addCell(row, text) {
    let cell = document.createElement("td");
    let cellText = document.createTextNode(text);
    cell.appendChild(cellText);
    row.appendChild(cell);
}

function addAddressCell(row, address) {
    var addr;
    switch (address.type) {
        case "BTC":
            addr = address.addr;
            break;
        case "ETH":
            addr = address.account;
            break;
        default:
            addr = "unknown";
    }
    let cell = document.createElement("td");
    let contentWrapper = document.createElement("div");
    contentWrapper.setAttribute("class", "address-cell");
    let copyBtn = document.createElement("a");
    let addressTextWrapper = document.createElement("span");
    let addressText = document.createTextNode(truncate(addr, 15));
    addressTextWrapper.appendChild(addressText);
    tippy(addressTextWrapper, {
        content: addr,
    });
    contentWrapper.appendChild(addressTextWrapper);
    copyBtn.setAttribute("class", "button clear icon-only");
    copyBtn.innerHTML = '<img src="/images/copy.svg?size=18"></img>';
    copyBtn.addEventListener("click", () => {
        navigator.clipboard.writeText(addr).then(function () { }, function (err) {
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
    contentWrapper.appendChild(copyBtn);
    cell.appendChild(contentWrapper);
    row.appendChild(cell);
}

function addRequestIdCell(row, requestId) {
    let cell = document.createElement("td");
    let requestIdTextWrapper = document.createElement("span");
    let requestIdText = document.createTextNode(requestId.substring(0, 8) + "...");
    requestIdTextWrapper.appendChild(requestIdText);
    tippy(requestIdTextWrapper, {
        content: requestId,
    });
    cell.appendChild(requestIdTextWrapper);
    row.appendChild(cell);
}

function addStatusCell(row, status) {
    let cell = document.createElement("td");
    let cellText;
    switch (status.type) {
        case "InProgress":
            cellText = document.createTextNode("In progress (" + status.confirmations + " of 2)");
            break;
        case "Confirmed":
            cellText = document.createTextNode("Confirmed");
            break;
        case "OpRejected":
            cellText = document.createTextNode("Rejected by operators");
            break;
        case "NodeRejected":
            cellText = document.createTextNode("Rejected by node (" + status.reason + ")");
            break;
        case "Completed":
            let linkToTx;
            switch (status.txid.type) {
                case "BTC":
                    linkToTx = `https://mempool.space/tx/${status.txid.txid}`
                    break;
                default:
                    console.error('undefined link for: ', status.txid.type);
                    break;
            }

            const goToExplorerLink = document.createElement("a");
            goToExplorerLink.href = linkToTx;
            goToExplorerLink.target = "_blank"
            goToExplorerLink.setAttribute("class", "button clear icon-only");
            goToExplorerLink.innerHTML = '<img src="/images/corner-right-up.svg?size=18"></img>';
            tippy(goToExplorerLink, {
                content: "View transaction in explorer"
            });
            statusText = document.createTextNode("Completed");
            const contentWrapper = document.createElement("div");
            contentWrapper.appendChild(statusText);
            contentWrapper.appendChild(goToExplorerLink);
            cellText = contentWrapper;
            break;
        default:
            cellText = document.createTextNode("Unknown");
    };
    cell.appendChild(cellText);
    row.appendChild(cell);
}

async function confirmRequest(confirmationData) {
    const response = await makeSignedRequest(confirmationData, 'confirm', 'POST');
    if (response.ok) {
        await updateWithdrawalRequests();
    } else {
        alert("Error: " + response.text);
    };
}

async function rejectRequest(confirmationData) {
    const response = await makeSignedRequest(confirmationData, 'reject', 'POST');;
    if (response.ok) {
        await updateWithdrawalRequests();
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

async function updateWithdrawalRequests() {
    clearWithdrawalRequests();
    const response = await makeSignedRequest(null, "request", 'GET');
    if (response.ok) {
        let data = await response.json();
        let sortedData = data.sort((a, b) => {
            let dateA = new Date(a.created_at);
            let dateB = new Date(b.created_at);
            return dateA - dateB;
        });
        for (let withdrawalRequest of sortedData) {
            let row = document.createElement("tr");
            addCell(row, withdrawalRequest.created_at);
            addRequestIdCell(row, withdrawalRequest.id);
            addCell(row, withdrawalRequest.user);
            addCell(row, withdrawalRequest.address.type);
            addAddressCell(row, withdrawalRequest.address);
            addCell(row, withdrawalRequest.amount);
            addStatusCell(row, withdrawalRequest.confirmation_status);
            addActionBtns(row, withdrawalRequest, withdrawalRequest.confirmation_status.type);
            withdrawalRequestsTableBody.appendChild(row);
        }
    };
    return response;
}

function truncate(input, maxLength) {
    if (input.length > maxLength) {
        let left = Math.ceil(maxLength / 2);
        let right = Math.floor(maxLength / 2);
        return input.substring(0, left) + '...' + input.substring(input.length - right, input.length);
    }
    return input;
};

async function getInvite(body) {
    return await makeSignedRequest(body, "invite/generate", "POST");
};

async function getMyInvites(){
    return await makeSignedRequest(null, "invite/listmy", "GET");
}

function mkCopyBtn(parent, value){
    const copyBtn = document.createElement("button");
    const spn = document.createElement("span");
    const i = document.createElement("i");
    i.classList.add("mdi","mdi-content-copy");
    spn.classList.add("icon");
    copyBtn.classList.add("button", "is-h3", "is-ghost", "font-gray");
    spn.appendChild(i);
    copyBtn.appendChild(spn);
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

function renderError(parent, errMsg){
    const errNode = document.createElement("h3");
    errNode.classList.add("text-error");
    errNode.innerHTML = "Error! " + errMsg;
    parent.append(errNode);
}

async function genInvite(){
    const inviteLabelField = document.getElementById("invite-label");
    const inviteDisplay = document.getElementById("invite-display");
    inviteDisplay.style.display = 'flex';
    const label = inviteLabelField.value;
    if (!label) {
        renderError(inviteDisplay, "Label should be non-empty!")
    } else {
        const body = {label: label}
        const inviteResp = await getInvite(body)
        if (inviteResp.ok) {
            let invite = await inviteResp.json();
            const invitesList = invitesListTemplate({invites: [invite], isList: false});
            inviteDisplay.innerHTML = invitesList;
            mkCopyBtn(inviteDisplay, invite.invite.invite);
        } else {
            renderError(inviteDisplay, JSON.stringify(inviteResp));
        }
    }
}

async function listInvites(){
    const invitesListBody = document.getElementById("invites-list");
    const invitesResp = await getMyInvites();
    if (invitesResp.ok){
        const invites = await invitesResp.json();
        const invitesList = invitesListTemplate({invites: invites, isList: true});
        invitesListBody.innerHTML = invitesList;
        const inviteDisplays = invitesListBody.querySelectorAll(".invite-display");
        inviteDisplays.forEach(display => {
            const value = display.querySelector('.invite').innerHTML;
            mkCopyBtn(display, value)
        })
    } else {
        const errMsg = document.createElement("h3");
        errMsg.classList.add("text-error");
        errMsg.innerHTML = "Error: " + invitesResp.status + ": " + invitesResp.statusText;
        invitesListBody.innerHTML = "";
        invitesListBody.append(errMsg)
    }

}

async function loadInvitesTab(){
    const inivitesDrawUpdate = invitesTemplate();
    invitesTabBody.innerHTML = inivitesDrawUpdate;

    withdrawsTabBody.style.display = 'none';
    withdrawsTab.classList.remove('is-active');
    invitesTabBody.style.display = 'block';
    invitesTab.classList.add('is-active');

    const genInviteBtn = document.getElementById("gen-invite-btn");
    const listInvitesBtn = document.getElementById("btn-list-invites");
    genInviteBtn.onclick = genInvite;
    listInvitesBtn.onclick = listInvites;
}

async function init(){
    const [invitesTemp, invitesListTemp] = await Promise.allSettled([
        await loadTemplate("/scripts/templates/invites.html.hbs"),
        await loadTemplate("/scripts/templates/inviteslist.html.hbs")
    ]);
    invitesTemplate = invitesTemp.value;
    invitesListTemplate = invitesListTemp.value;
    fileSelector.addEventListener('change', loadKeyFile);
    invitesTab.onclick = loadInvitesTab;
}

document.addEventListener("DOMContentLoaded", init);