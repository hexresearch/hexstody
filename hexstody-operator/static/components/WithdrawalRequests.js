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
            <currency-select @currency-selected="setCurrency" />
            <hot-wallet-balance :currency="currency" />
            <withdrawal-requests-table :currency="currency" />
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
}