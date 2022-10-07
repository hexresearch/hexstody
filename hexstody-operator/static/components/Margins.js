import { getSupportedCurrencies, getCurrencyName, formatCurrencyValue, getPairRate, getMargin, isNumeric, setMargin } from "../scripts/common.js"

export const MarginsTab = {
    template:
        /*html*/
        `<div class="flex-column">
            <h4 class="m-0;">Select pair</h4>
            <div style="display: flex;">
                <div class="mr-2em">
                    <span>From:</span>
                    <select id="currency_from" class="mr-2em" v-model="currency_from">
                        <option v-for="cur in currencies" :value="cur">{{getCurrencyName(cur)}}</option>
                    </select>
                </div>
                <div class="mr-2em">
                    <span>To:</span>
                    <select id="currency_to" class="mr-2em" v-model="currency_to">
                        <option v-for="cur in currencies" :value="cur">{{getCurrencyName(cur)}}</option>
                    </select>
                </div>
                <div class="mr-2em">
                    <span>Rate</span>
                    <h4 v-if='isLoading' class="flex-row m-0">
                        Loading
                        <div class="loading-circle"></div>
                    </h4>
                    <h4 v-else-if="isRateLoaded" class="m-0">
                        {{rate.rate}} {{getCurrencyName(currency_to)}}
                    </h4>
                    <h4 v-else class="text-error m-0">
                        {{rate}}
                    </h4>
                </div>
                <div class="mr-2em">
                    <span>Margin(%):</span>
                    <input type="text" id="margin-input" v-model="marginField">
                </div>
                <div v-if='canSet' style="display: flex;">
                    <button class="button mt-auto" @click='setBtnClick()'>Set</button>
                </div>
            </div>
        </div>`,
    data() {
        return {
            isLoading: false,
            currencies: [],
            currency_from: null,
            currency_to: null,
            rate: null,
            margin: null,
            marginField: null,
        }
    },
    methods: {
        getCurrencyName,
        formatCurrencyValue,
        async fetchData() {
            this.isLoading = true
            const currencies = await getSupportedCurrencies(this.privateKeyJwk, this.publicKeyDer, this.currency).then(r => r.json())
            this.currencies = currencies
            this.isLoading = false
            if (currencies.length >= 2) {
                this.currency_from = currencies[0];
                this.currency_to = currencies[1];
            }
        },
        async setBtnClick(){
            const margin = parseFloat(this.marginField)
            const req = {
                currency_from: this.currency_from, 
                currency_to: this.currency_to, 
                margin: margin.toFixed(1)
            };
            console.log(req)
            await setMargin(this.privateKeyJwk, this.publicKeyDer, req)
        },
        async loadPairData(){
            this.isLoading = true;
            this.rate = null;
            this.margin = null;
            if (this.currency_from && this.currency_to) {
                const rate = await getPairRate(this.currency_from, this.currency_to).then(r => r.json());
                console.log(rate);
                this.rate = rate
                const margin = await getMargin(this.currency_from, this.currency_to).then(r => r.json());
                this.margin = margin.margin;
                this.marginField = margin.margin;
            }
            this.isLoading = false;
        }
    },
    watch: {
        currency_from(){
            this.loadPairData()
        },
        currency_to(){
            this.loadPairData()
        }
    },
    computed: {
        isRateLoaded() {
            if (typeof this.rate === 'object' && this.rate !== null && "rate" in this.rate) {
                return true
            } else {
                return false
            }
        },
        canSet(){
            return this.margin != this.marginField 
                && isNumeric(this.marginField)
                && this.currency_from != this.currency_to
        }
    },
    async created() {
        await this.fetchData()
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
    }
}