import { getHotWalletBalance, formatCurrencyValue } from "../scripts/common.js"

export const HotWalletBalance = {
    template:
        /*html*/
        `<div class="flex-row">
            <h4>Hot wallet balance:</h4>
            <h4 v-if='isLoading' class="flex-row">
                Loading
                <div class="loading-circle"></div>
            </h4>
            <h4 v-else-if="isBalanceLoaded">
                {{formatCurrencyValue(currency, balance.balance)}} {{currency}}
            </h4>
            <h4 v-else class="text-error">
                {{balance}}
            </h4>
        </div>`,
    methods: {
        async fetchData() {
            this.isLoading = true
            const response = await getHotWalletBalance(this.privateKeyJwk, this.publicKeyDer, this.currency)
            this.balance = await response.json()
            this.isLoading = false
        },
        formatCurrencyValue
    },
    data() {
        return {
            balance: null,
            isLoading: false
        }
    },
    computed: {
        isBalanceLoaded() {
            if (typeof this.balance === 'object' && this.balance !== null && "balance" in this.balance) {
                return true
            } else {
                return false
            }
        }
    },
    watch: {
        currency: 'fetchData'
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
    },
}