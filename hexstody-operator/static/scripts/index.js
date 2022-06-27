const fileSelector = document.getElementById("file-selector");
const fileSelectorStatus = document.getElementById("file-selector-status");
const withdrawalRequestsTableBody = document.getElementById("withdrawal-requests-table-body");

let privateKeyJwk;
let publicKeyDer;

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
            return;
        };
    };
    if (!keyObj.isPrivate) {
        fileSelectorStatus.className = "text-error";
        fileSelectorStatus.innerText = "The selected key is not private";
        clearWithdrawalRequests();
        return;
    } else {
        try {
            privateKeyJwk = await keyObj.export('jwk');
            publicKeyDer = await keyObj.export('der', { outputPublic: true, compact: true });
        } catch (error) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = error;
            clearWithdrawalRequests();
            return;
        };
    };
    const response = await updateWithdrawalRequests();
    if (response.ok) {
        fileSelectorStatus.className = "text-success";
        fileSelectorStatus.innerText = "Private key imported successfully!";
        // // Debug fucntion to create request
        // makeSignedRequest({
        //     user: "Bob",
        //     address: {
        //         type: "BTC",
        //         addr: "1BNwxHGaFbeUBitpjy2AsKpJ29Ybxntqvb"
        //     },
        //     amount: 42
        // },
        //     "request",
        //     "POST"
        // )
    } else {
        if (response.status == 403) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = "Invalid key";
            clearWithdrawalRequests();
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
    let cell = document.createElement("td");
    let contentWrapper = document.createElement("div");
    contentWrapper.setAttribute("class", "address-cell");
    let copyBtn = document.createElement("a");
    let addressTextWrapper = document.createElement("span");
    let addressText = document.createTextNode(truncate(address, 15));
    addressTextWrapper.appendChild(addressText);
    tippy(addressTextWrapper, {
        content: address,
    });
    contentWrapper.appendChild(addressTextWrapper);
    copyBtn.setAttribute("class", "button clear icon-only");
    copyBtn.innerHTML = '<img src="https://icongr.am/feather/copy.svg?size=18"></img>';
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
        case "Rejected":
            cellText = document.createTextNode("Rejected");
            break;
        default:
            cellText = document.createTextNode("Unknown");
    };
    cell.appendChild(cellText);
    row.appendChild(cell);
}

async function confirmRequest(withdrawalRequest) {
    const response = await makeSignedRequest(withdrawalRequest, 'confirm', 'POST');
    if (response.ok) {
        await updateWithdrawalRequests();
    } else {
        alert("Error: " + response.text);
    };
}

async function rejectRequest(withdrawalRequest) {
    const response = await makeSignedRequest(withdrawalRequest, 'reject', 'POST');;
    if (response.ok) {
        await updateWithdrawalRequests();
    } else {
        alert("Error: " + response.text);
    };
}

function addActionBtns(row, withdrawalRequest, requestStatus) {
    let disabled = (requestStatus == "InProgress") ? false : true;
    let cell = document.createElement("td");
    let btnRow = document.createElement("div");
    btnRow.setAttribute("class", "row pull-left");

    let confirmBtnCol = document.createElement("div");
    confirmBtnCol.setAttribute("class", "col");
    let confirmBtn = document.createElement("button");
    confirmBtn.addEventListener("click", () => { confirmRequest(withdrawalRequest); });
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
    rejectBtn.addEventListener("click", () => { rejectRequest(withdrawalRequest); });
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
            addAddressCell(row, withdrawalRequest.address.addr);
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

window.addEventListener('load', function () {
    fileSelector.addEventListener('change', loadKeyFile);
});
