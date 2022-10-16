import { getFeeEstimates, postFeeEstimates } from "../scripts/common.js"

export const FeesTab = {
    template:
        /*html*/
        `<div class="flex-column">
        <input type="text" id="margin-input" v-model="btc_bytes_per_tx">
        <input type="text" id="margin-input" v-model="eth_tx_gas_limit">
        <input type="text" id="margin-input" v-model="erc20_tx_gas_limit">
        <button class="button mt-auto" @click='setBtnClick()'>Set</button>
        </div>`,
    data() {
        return {
            btc_bytes_per_tx: 0,
            eth_tx_gas_limit: 0,
            erc20_tx_gas_limit: 0,
        }
    },
    methods: {
        async met() {
            return getFeeEstimates(this.privateKeyJwk, this.publicKeyDer)
        },
        async setBtnClick() {
            const request = {
                btc_bytes_per_tx: Number.parseInt(this.btc_bytes_per_tx),
                eth_tx_gas_limit: Number.parseInt(this.eth_tx_gas_limit),
                erc20_tx_gas_limit: Number.parseInt(this.erc20_tx_gas_limit),
            }
            await postFeeEstimates(this.privateKeyJwk, this.publicKeyDer, request)
            alert("wasd")
        }

    },
    watch: {
    },
    computed: {
    },
    async created() {
        const r = await this.met().then(r => r.json())
        this.btc_bytes_per_tx = r.btc_bytes_per_tx
        this.eth_tx_gas_limit = r.eth_tx_gas_limit
        this.erc20_tx_gas_limit = r.erc20_tx_gas_limit
        console.log("test")
    },
    inject: ['privateKeyJwk', 'publicKeyDer'],
}