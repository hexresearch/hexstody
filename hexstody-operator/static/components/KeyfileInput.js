import { getSupportedCurrencies } from "../scripts/common.js"

export const KeyfileComponent = {
    template:
        /*html*/
        `<div class="mb-1">
            <h4>Select the private key file</h4>
            <div class="flex-row keyfile-input-wrapper">
                <div class="flex-row">
                    <input class="keyfile-input" type="file" accept=".pem" @change="loadKeyFile" ref="fileInput"/>
                    <div v-if="isLoading" class="flex-row">
                        <span>Loading</span>
                        <div class="loading-circle"></div>
                    </div>
                    <span v-else :class="[{ 'text-error': hasError }, { 'text-success': !hasError }]">
                        {{ statusMessage }}
                    </span>
                </div>
                <button v-if="loggedIn" type="button" class="button outline icon" @click="logout">
                    Log out
                    <span class="mdi mdi-logout ml-1"></span>
                </button>
            </div>
        </div>`,
    data() {
        return {
            isLoading: false,
            hasError: false,
            statusMessage: "",
            loggedIn: false,
        }
    },
    methods: {
        loadKeyFile(event) {
            const reader = new FileReader()
            if (event.target.files.length > 0) {
                this.clearKey()
                this.isLoading = true
                const file = event.target.files.item(0)
                reader.readAsText(file)
            }
            reader.onload = async () => {
                await this.importKey(reader.result)
                this.isLoading = false
            }
            reader.onerror = () => {
                this.hasError = true
                this.statusMessage = reader.error
                this.isLoading = false
            }
        },
        async importKey(keyText) {
            const keyObj = new window.jscu.Key("pem", keyText)
            if (keyObj.isEncrypted) {
                const password = window.prompt("Enter password")
                try {
                    await keyObj.decrypt(password)
                } catch (_error) {
                    this.hasError = true
                    this.statusMessage = "Wrong password"
                    return
                }
            }
            if (!keyObj.isPrivate) {
                this.hasError = true
                this.statusMessage = "The selected key is not private"
                return
            }
            let privateKeyJwk
            let publicKeyDer
            try {
                privateKeyJwk = await keyObj.export("jwk")
                publicKeyDer = await keyObj.export("der", {
                    outputPublic: true,
                    compact: true,
                })
            } catch (error) {
                this.hasError = true
                this.statusMessage = error
                return
            }
            this.testKey(privateKeyJwk, publicKeyDer)
        },
        // Tests key by sending signed request to server
        async testKey(privateKeyJwk, publicKeyDer) {
            // We use 'getSupportedCurrencies' function here that calls '/currency' endpoint internally 
            // to test that the key is valid, but it could be any other endpoint that uses key-based auth.
            const response = await getSupportedCurrencies(privateKeyJwk, publicKeyDer)
            if (response.ok) {
                this.statusMessage = "Private key imported successfully"
                this.$emit('key-imported', privateKeyJwk, publicKeyDer)
                this.loggedIn = true
            } else {
                if (response.status == 403) {
                    this.hasError = true
                    this.statusMessage = "Invalid key"
                    return
                } else {
                    this.hasError = true
                    this.statusMessage = response.text()
                    return
                }
            };
        },
        clearKey() {
            this.hasError = false
            this.statusMessage = ""
            this.$emit('key-reset')
            this.loggedIn = false
        },
        logout() {
            this.clearKey()
            this.$refs.fileInput.value = null
        }
    },
}
