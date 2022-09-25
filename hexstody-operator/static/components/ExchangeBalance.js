import { getExchangeBalances, formatCurrencyValue, getCurrencyName } from "../scripts/common.js"

export const ExchangeBalance = {
    template:
        /*html*/
        `<div class="flex-row">
            <h4>Exchange balance:</h4>
            <h4 v-if='isLoading' class="flex-row">
                Loading
                <div class="loading-circle"></div>
            </h4>
            <h4 v-else-if="isBalanceLoaded">
                 {{formatCurrencyValue(currency, balance.balance)}} {{getCurrencyName(currency)}}
            </h4>
            <h4 v-else class="text-error">
                {{balance}}
            </h4>
        </div>`,
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
    methods: {
        formatCurrencyValue,
        getCurrencyName,
        async fetchData() {
            this.isLoading = true
            const response = await getExchangeBalances(this.privateKeyJwk, this.publicKeyDer)
            const balances = await response.json()
            this.balance = balances.find(({ currency }) => getCurrencyName(currency) === getCurrencyName(this.currency))
            this.isLoading = false
        },
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
    }
}