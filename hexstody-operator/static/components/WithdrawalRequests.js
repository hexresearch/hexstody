import { CurrencySelect } from "./CurrencySelect.js"
import { HotWalletBalance } from "./HotWalletBalance.js"
import { WithdrawalRequestsTable } from "./WithdrawalRequestsTable.js"

export const WithdrawalRequests = {
    components: {
        CurrencySelect,
        HotWalletBalance,
        WithdrawalRequestsTable
    },
    template:
        /*html*/
        `<div>
            <currency-select :private-key-jwk="privateKeyJwk" :public-key-der="publicKeyDer" @currency-selected="setCurrency" />
            <hot-wallet-balance :private-key-jwk="privateKeyJwk" :public-key-der="publicKeyDer" :currency="currency" />
            <withdrawal-requests-table :private-key-jwk="privateKeyJwk" :public-key-der="publicKeyDer" :currency="currency" />
        </div>`,
    data() {
        return {
            currency: null,
        }
    },
    methods: {
        setCurrency(currency) {
            this.currency = currency
        },
    },
    props: {
        privateKeyJwk: {
            type: Object,
            required: true
        },
        publicKeyDer: {
            type: Object,
            required: true
        }
    }
}