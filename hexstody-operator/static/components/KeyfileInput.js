app.component('keyfile-input', {
    template:
        /*html*/
        `<div id="keyfile-wrapper" class="mb-1">
            <h4>Select the private key file</h4>
            <div class="keyfile-input-wrapper">
                <input type="file" id="keyfile-input" accept=".pem" @change="loadKeyFile"/>
                <span v-if="!isLoading" id="keyfile-input-status" :class="[{ 'text-error': hasError }, { 'text-success': !hasError }]">{{ statusMessage }}</span>
                <div v-if="isLoading" class="keyfile-input-wrapper">
                    <span>Loading...</span>
                    <div class="loading-circle"></div>
                </div> 
            </div>
        </div>`,
    data() {
        return {
            isLoading: false,
            hasError: false,
            statusMessage: '',
            privateKeyJwk: null,
            publicKeyDer: null,
        }
    },
    methods: {
        loadKeyFile(event) {
            this.clearKey();
            this.isLoading = true;
            const file = event.target.files.item(0);
            const reader = new FileReader();
            reader.readAsText(file);
            reader.onload = async () => {
                await this.importKey(reader.result);
                this.isLoading = false;
            };
            reader.onerror = () => {
                this.hasError = true;
                this.statusMessage = reader.error;
                this.isLoading = false;
            };
        },
        async importKey(keyText) {
            const keyObj = new window.jscu.Key('pem', keyText);
            if (keyObj.isEncrypted) {
                const password = window.prompt("Enter password");
                try {
                    await keyObj.decrypt(password);
                } catch (_error) {
                    this.hasError = true;
                    this.statusMessage = "Wrong password";
                    return;
                };
            };
            if (!keyObj.isPrivate) {
                this.hasError = true;
                this.statusMessage = "The selected key is not private";
                return;
            };
            try {
                this.privateKeyJwk = await keyObj.export('jwk');
                this.publicKeyDer = await keyObj.export('der', { outputPublic: true, compact: true });
            } catch (error) {
                this.hasError = "text-error";
                this.statusMessage = error;
                return;
            };
            this.testKey();
        },
        // Tests key by sending signed request to server
        async testKey() {
            const response = await getSupportedCurrencies();
            console.log(response);
            // if (response.ok) {
            //     fileSelectorStatus.className = "text-success";
            //     fileSelectorStatus.innerText = "Private key imported successfully";
            //     // Here we test the key and get the list of supported currencies
            //     const currencies = await response.json();
            //     await loadAuthorizedContent(currencies);
            // } else {
            //     if (response.status == 403) {
            //         fileSelectorStatus.className = "text-error";
            //         fileSelectorStatus.innerText = "Invalid key";
            //         clearAuthorizedWrapper();
            //         return;
            //     } else {
            //         fileSelectorStatus.className = "text-error";
            //         fileSelectorStatus.innerText = response.text();
            //         clearAuthorizedWrapper();
            //     }
            // };
        },
        clearKey() {
            this.hasError = false;
            this.statusMessage = '';
            this.privateKeyJwk = null;
            this.publicKeyDer = null;
        }
    },
})