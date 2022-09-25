import { CurrencySelect } from "./CurrencySelect.js"
import { ExchangeBalance } from "./ExchangeBalance.js"
import { ExchangeDepositAddress } from "./ExchangeDepositAddress.js"
import { ExchangeRequestsTable } from "./ExchangeRequestsTable.js"

export const ExchangeRequests = {
    components: {
        CurrencySelect,
        ExchangeBalance,
        ExchangeDepositAddress,
        ExchangeRequestsTable,
    },
    template:
        /*html*/
        `<div>
            <currency-select :private-key-jwk="privateKeyJwk" :public-key-der="publicKeyDer" @currency-selected="setCurrency" />
            <exchange-balance :private-key-jwk="privateKeyJwk" :public-key-der="publicKeyDer" :currency="currency" />
            <exchange-deposit-address :private-key-jwk="privateKeyJwk" :public-key-der="publicKeyDer" :currency="currency" />
            <exchange-requests-table :private-key-jwk="privateKeyJwk" :public-key-der="publicKeyDer" :currency="currency" />
        </div>`,
    methods: {
        setCurrency(currency) {
            this.currency = currency
        },
    },
    data() {
        return {
            currency: null,
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
        }
    }
}