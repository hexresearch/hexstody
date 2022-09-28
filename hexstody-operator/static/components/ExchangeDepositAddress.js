import {
    getExchangeDepositAddress,
} from "../scripts/common.js"

export const ExchangeDepositAddress = {
    template:
        /*html*/
        `<div :class="{ 'flex-row': isLoading }">
            <h4>Exchange address</h4>
            <h4 v-if='isLoading' class="flex-row">
                Loading
                <div class="loading-circle"></div>
            </h4>
            <div v-else-if="isAddressLoaded">
                <img :src="qrCode(address.qr_code_base64)">
                <p>{{address.address}}</p>
            </div>
            <h4 v-else class="text-error">
                {{address}}
            </h4>
        </div>`,
    methods: {
        async fetchData() {
            this.isLoading = true
            const exchangeDesositAddressResponse = await getExchangeDepositAddress(this.privateKeyJwk, this.publicKeyDer, this.currency)
            this.address = await exchangeDesositAddressResponse.json()
            this.isLoading = false
        },
        qrCode(text) {
            return `data:image/png;base64, ${text}`
        },
    },
    data() {
        return {
            address: null,
            isLoading: false
        }
    },
    watch: {
        currency: 'fetchData'
    },
    computed: {
        isAddressLoaded() {
            if (typeof this.address === 'object' && this.address !== null && "address" in this.address) {
                return true
            } else {
                return false
            }
        }
    },
    props: {
        privateKeyJwk: {
            type: Object,
            required: true
        },
        publicKeyDer: {
            type: Object,
            required: true
        },
        currency: {}
    }
}
