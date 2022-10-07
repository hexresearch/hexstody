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
            <currency-select @currency-selected="setCurrency" />
            <exchange-balance :currency="currency" />
            <exchange-deposit-address :currency="currency" />
            <exchange-requests-table :currency="currency" />
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
}