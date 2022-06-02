const fileSelector = document.getElementById("file-selector");
const fileSelectorStatus = document.getElementById("file-selector-status");
const withdrawalRequestsTable = document.getElementById("withdrawal-requests-table");

let privateKeyJwk;
let publicKeyJwk;

/*
Convert a string into an ArrayBuffer
from https://developers.google.com/web/updates/2012/06/How-to-convert-ArrayBuffer-to-and-from-String
*/
function str2ab(str) {
    const buf = new ArrayBuffer(str.length);
    const bufView = new Uint8Array(buf);
    for (let i = 0, strLen = str.length; i < strLen; i++) {
        bufView[i] = str.charCodeAt(i);
    }
    return buf;
}

function enableActionButtons() {
    let actionButtons = document.querySelectorAll("#withdrawal-requests-table button");
    for (let i = 0; i < actionButtons.length; i++) {
        actionButtons[i].disabled = false;
    }
}

function disableActionButtons() {
    let actionButtons = document.querySelectorAll("#withdrawal-requests-table button");
    for (let i = 0; i < actionButtons.length; i++) {
        actionButtons[i].disabled = true;
    }
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
            disableActionButtons();
            return;
        };
    };
    if (!keyObj.isPrivate) {
        fileSelectorStatus.className = "text-error";
        fileSelectorStatus.innerText = "The selected key is not private!";
        disableActionButtons();
        return;
    } else {
        try {
            privateKeyJwk = await keyObj.export('jwk');
            publicKeyJwk = await keyObj.export('jwk', { outputPublic: true });
            fileSelectorStatus.className = "text-success";
            fileSelectorStatus.innerText = "Private key imported successfully!";
            enableActionButtons();
        } catch (error) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = error;
            disableActionButtons();
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

async function processRequest(request, isConfirmation) {
    const url = (isConfirmation ? "/confirm/" : "/reject/") + request.id;
    const requestBody = JSON.stringify(request);
    const nonce = Date.now();
    const publicKeyStr = JSON.stringify(publicKeyJwk);
    const elements = [url, requestBody, nonce, publicKeyStr];
    const msg = elements.join(':');
    const binaryMsg = str2ab(msg);
    const signature = await window.jscec.sign(binaryMsg, privateKeyJwk, 'SHA-256').catch(error => {
        alert(error);
    });
    try {
        await fetch(url, {
            method: 'POST',
            body: requestBody,
            headers: {
                'Content-Type': 'application/json',
                'Signarute': btoa(signature),
                'Signarute-Nonce': btoa(nonce),
                'Signature-Public-Key': publicKeyStr
            }
        });
    } catch (error) {
        alert('Error:', error);
    }
}

async function updateWithdrawalRequests() {
    async function getWithdrawalRequests() {
        const response = await fetch("/request");
        return response.json();
    }

    const data = await getWithdrawalRequests();
    let tableBody = document.createElement("tbody");

    function addCell(row, text) {
        let cell = document.createElement("td");
        let cellText = document.createTextNode(text);
        cell.appendChild(cellText);
        row.appendChild(cell);
    }

    function addActionBtns(row, request) {
        let cell = document.createElement("td");
        let btnRow = document.createElement("div");
        btnRow.setAttribute("class", "row");

        let confirmBtnCol = document.createElement("div");
        confirmBtnCol.setAttribute("class", "col");
        let confirmBtn = document.createElement("button");
        confirmBtn.addEventListener("click", () => processRequest(request, true));
        let confirmBtnText = document.createTextNode("Confirm")
        confirmBtn.appendChild(confirmBtnText);
        confirmBtn.setAttribute("class", "button primary");
        confirmBtn.setAttribute("disabled", "");
        confirmBtnCol.appendChild(confirmBtn);
        btnRow.appendChild(confirmBtnCol);

        let rejectBtnCol = document.createElement("div");
        rejectBtnCol.setAttribute("class", "col");
        let rejectBtn = document.createElement("button");
        rejectBtn.addEventListener("click", () => processRequest(request, false));
        let rejectBtnText = document.createTextNode("Reject")
        rejectBtn.appendChild(rejectBtnText);
        rejectBtn.setAttribute("class", "button error");
        rejectBtn.setAttribute("disabled", "");
        rejectBtnCol.appendChild(rejectBtn);
        btnRow.appendChild(rejectBtnCol);

        cell.appendChild(btnRow);
        row.appendChild(cell);
    }

    for (let request of data) {
        let row = document.createElement("tr");
        let currency = Object.keys(request["address"])[0];
        addCell(row, request["id"]);
        addCell(row, request["user"]);
        addCell(row, currency);
        addCell(row, request["address"][currency]);
        addCell(row, request["created_at"]);
        addCell(row, request["amount"]);
        addCell(row, request["confirmation_status"]);
        addActionBtns(row, request);
        tableBody.appendChild(row);
    }
    withdrawalRequestsTable.appendChild(tableBody);
}

window.addEventListener('load', function () {
    fileSelector.addEventListener('change', loadKeyFile);
});

window.onload = updateWithdrawalRequests;
