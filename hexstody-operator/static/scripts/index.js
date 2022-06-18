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
        fileSelectorStatus.innerText = "The selected key is not private!";
        clearWithdrawalRequests();
        return;
    } else {
        try {
            privateKeyJwk = await keyObj.export('jwk');
            publicKeyDer = await keyObj.export('der', { outputPublic: true, compact: true });
            fileSelectorStatus.className = "text-success";
            fileSelectorStatus.innerText = "Private key imported successfully!";
            await updateWithdrawalRequests();

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

        } catch (error) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = error;
            clearWithdrawalRequests();
            return;
        };
    };
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
    try {
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
    } catch (error) {
        alert('Error:', error);
    }
}

function clearWithdrawalRequests() {
    withdrawalRequestsTableBody.textContent = '';
}

async function updateWithdrawalRequests() {
    async function getWithdrawalRequests() {
        const response = await makeSignedRequest(null, "request", 'GET');
        let res = await response.json()
        return res;
    }

    clearWithdrawalRequests();

    const data = await getWithdrawalRequests();

    function addCell(row, text) {
        let cell = document.createElement("td");
        let cellText = document.createTextNode(text);
        cell.appendChild(cellText);
        row.appendChild(cell);
    }

    function addActionBtns(row, withdrawal_request) {
        let cell = document.createElement("td");
        let btnRow = document.createElement("div");
        btnRow.setAttribute("class", "row");

        let confirmBtnCol = document.createElement("div");
        confirmBtnCol.setAttribute("class", "col");
        let confirmBtn = document.createElement("button");

        confirmBtn.addEventListener("click", () => makeSignedRequest(withdrawal_request, 'confirm', 'POST'));
        let confirmBtnText = document.createTextNode("Confirm")
        confirmBtn.appendChild(confirmBtnText);
        confirmBtn.setAttribute("class", "button primary");
        confirmBtnCol.appendChild(confirmBtn);
        btnRow.appendChild(confirmBtnCol);

        let rejectBtnCol = document.createElement("div");
        rejectBtnCol.setAttribute("class", "col");
        let rejectBtn = document.createElement("button");

        rejectBtn.addEventListener("click", () => makeSignedRequest(withdrawal_request, 'reject', 'POST'));
        let rejectBtnText = document.createTextNode("Reject")
        rejectBtn.appendChild(rejectBtnText);
        rejectBtn.setAttribute("class", "button error");
        rejectBtnCol.appendChild(rejectBtn);
        btnRow.appendChild(rejectBtnCol);

        cell.appendChild(btnRow);
        row.appendChild(cell);
    }

    for (let withdrawal_request of data) {
        let row = document.createElement("tr");
        addCell(row, withdrawal_request.id);
        addCell(row, withdrawal_request.user);
        addCell(row, withdrawal_request.address.type);
        addCell(row, withdrawal_request.address.addr);
        addCell(row, withdrawal_request.created_at);
        addCell(row, withdrawal_request.amount);
        addCell(row, withdrawal_request.confirmation_status.type);
        addActionBtns(row, withdrawal_request);
        withdrawalRequestsTableBody.appendChild(row);
    }
}

window.addEventListener('load', function () {
    fileSelector.addEventListener('change', loadKeyFile);
});
