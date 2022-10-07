import { KeyfileComponent } from "../components/KeyfileInput.js"
import { AuthorizedContent } from "../components/AuthorizedContent.js"

const app = Vue.createApp({
    template:
        /*html*/
        `<div class="container">
            <keyfile-input @key-imported="setKey" @key-reset="resetKey" />
            <authorized-content v-if="isAuthorized" />
        </div>`,
    data() {
        return {
            privateKeyJwk: null,
            publicKeyDer: null,
            eventToggle: false
        }
    },
    provide() {
        return {
            eventToggle: Vue.computed(() => this.eventToggle),
            privateKeyJwk: Vue.computed(() => this.privateKeyJwk),
            publicKeyDer: Vue.computed(() => this.publicKeyDer)
        }
    },
    created() {
        const eventSource = new EventSource('/state-updates-events')
        eventSource.onmessage = (_event) => {
            this.eventToggle = !this.eventToggle
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

// Remove this when Vue.js v3.3 comes out
app.config.unwrapInjectedRef = true

const mountedApp = app.mount('#app')
