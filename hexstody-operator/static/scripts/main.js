import { KeyfileComponent } from "../components/KeyfileInput.js"
import { AuthorizedContent } from "../components/AuthorizedContent.js"

const app = Vue.createApp({
    template:
        /*html*/
        `<div class="container">
            <keyfile-input @key-imported="setKey" @key-reset="resetKey"></keyfile-input>
            <authorized-content v-if="isAuthorized" :private-key-jwk="privateKeyJwk" :public-key-der="publicKeyDer">
            </authorized-content>
        </div>`,
    data() {
        return {
            privateKeyJwk: null,
            publicKeyDer: null
        }
    },
    methods: {
        setKey(privateKeyJwk, publicKeyDer) {
            this.privateKeyJwk = privateKeyJwk
            this.publicKeyDer = publicKeyDer
        },
        resetKey() {
            this.privateKeyJwk = null
            this.publicKeyDer = null
        }
    },
    computed: {
        isAuthorized() {
            return (this.privateKeyJwk && this.publicKeyDer) ? true : false
        }
    }
})

app.component("keyfile-input", KeyfileComponent)
app.component("authorized-content", AuthorizedContent)

app.use(TippyVue)

const mountedApp = app.mount('#app')
