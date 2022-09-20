import { getSupportedCurrencies, getCurrencyName } from "../scripts/common.js"

export const CurrencySelect = {
    template:
        /*html*/
        `<div class="flex-row currency-select-wrapper">
            <h4>Select currency: </h4>
            <select @change="setCurrency">
                <option v-for="(currency, index) in currencies" :key="currency" :value="index">
                    {{ getCurrencyName(currency) }}
                </option>
            </select>
        </div>`,
    data() {
        return {
            currencies: []
        }
    },
    async created() {
        await this.fetchData()
        this.$emit('currency-selected', this.currencies[0])
    },
    methods: {
        getCurrencyName,
        async fetchData() {
            const response = await getSupportedCurrencies(this.privateKeyJwk, this.publicKeyDer)
            this.currencies = await response.json()
        },
        setCurrency(event) {
            this.$emit('currency-selected', this.currencies[event.target.value])
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
