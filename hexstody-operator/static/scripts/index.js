async function importKey(_event) {
    const fileSelectorStatus = document.getElementById("file-selector-status");
    const keyObj = new window.jscu.Key('pem', this.result);
    if (keyObj.isEncrypted) {
        const password = window.prompt("Enter password");
        try {
            await keyObj.decrypt(password);
        } catch (_error) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = "Wrong password";
            return;
        };
    };
    if (!keyObj.isPrivate) {
        fileSelectorStatus.className = "text-error";
        fileSelectorStatus.innerText = "The selected key is not private!";
        return;
    } else {
        try {
            let privateKeyJwk = await keyObj.export('jwk')
            fileSelectorStatus.className = "text-success";
            fileSelectorStatus.innerText = "Private key imported successfully!";
            // const msg = new Uint8Array(32);
            // for (let i = 0; i < 32; i++) msg[i] = 0xFF & i;

            // const sig = await window.jscec.sign(msg, privateKeyJwk, 'SHA-256').catch(error => {
            //     fileSelectorStatus.className = "text-error";
            //     fileSelectorStatus.innerText = error;
            // });
        } catch (error) {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = error;
            return;
        };
    };
}

async function loadKeyFile(event) {
    const fileSelectorStatus = document.getElementById("file-selector-status");
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

window.addEventListener('load', function () {
    const fileSelector = document.getElementById("file-selector");
    fileSelector.addEventListener('change', loadKeyFile);
});
