window.addEventListener('load', function () {

    const fileSelector = document.getElementById("file-selector");
    const fileSelectorStatus = document.getElementById("file-selector-status");

    fileSelector.addEventListener('change', async (event) => {
        fileSelectorStatus.className = "";
        fileSelectorStatus.innerText = "Loading...";
        const file = event.target.files.item(0);
        const reader = new FileReader();
        reader.readAsText(file);
        reader.onload = async () => {
            const privateKeyObj = new window.jscu.Key('pem', reader.result);
            if (!privateKeyObj.isPrivate) {
                fileSelectorStatus.className = "text-error";
                fileSelectorStatus.innerText = "The selected key is not private!";
            };
            if (privateKeyObj.isEncrypted) {
                const password = window.prompt("Enter password");
                await privateKeyObj.decrypt(password).catch(_ => {
                    fileSelectorStatus.className = "text-error";
                    fileSelectorStatus.innerText = "Wrong password";
                });
            };
            if (!privateKeyObj.isEncrypted && privateKeyObj.isPrivate) {
                fileSelectorStatus.className = "text-success";
                fileSelectorStatus.innerText = "Private key imported successfully!";
            };

            // const msg = new Uint8Array(32);
            // for (let i = 0; i < 32; i++) msg[i] = 0xFF & i;
            // let privateKeyJwk = await privateKeyObj.export('jwk');

            // const sig = await window.jscec.sign(msg, privateKeyJwk, 'SHA-256').catch(error => {
            //     fileSelectorStatus.className = "text-error";
            //     fileSelectorStatus.innerText = error;
            // });

        };
        reader.onerror = function () {
            fileSelectorStatus.className = "text-error";
            fileSelectorStatus.innerText = reader.error;
        };
    });

});
