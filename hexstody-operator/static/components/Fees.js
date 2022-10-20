import { getFeeEstimates, postFeeEstimates } from "../scripts/common.js"

export const FeesTab = {
    template:
        /*html*/
        `<div class="flex-column">
            <div style="display: flex;">
                <div class="mr-2em">
                    <span>BTC bytes per transaction:</span>
                    <input type="text" id="margin-input" v-model="btc_bytes_per_tx">
                </div>
                <div class="mr-2em">
                    <span>ETH transaction gas limit:</span>
                    <input type="text" id="margin-input" v-model="eth_tx_gas_limit">
                </div>
                <div class="mr-2em">
                    <span>ERC20 transaction gas limit:</span>
                    <input type="text" id="margin-input" v-model="erc20_tx_gas_limit">
                </div>
                <div style="display: flex;">
                    <button class="button mt-auto" @click='setBtnClick()'>Set</button>
                </div>
            </div>
        </div>`,
    data() {
        return {
            btc_bytes_per_tx: 0,
            eth_tx_gas_limit: 0,
            erc20_tx_gas_limit: 0,
        }
    },
    methods: {
        async setBtnClick() {
            const request = {
                btc_bytes_per_tx: Number.parseInt(this.btc_bytes_per_tx),
                eth_tx_gas_limit: Number.parseInt(this.eth_tx_gas_limit),
                erc20_tx_gas_limit: Number.parseInt(this.erc20_tx_gas_limit),
            }
            await postFeeEstimates(this.privateKeyJwk, this.publicKeyDer, request)
        }
    },
    watch: {
    },
    computed: {
    },
    async created() {
        const currentValues = await getFeeEstimates(this.privateKeyJwk, this.publicKeyDer)
            .then(r => r.json())
        this.btc_bytes_per_tx = currentValues.btc_bytes_per_tx
        this.eth_tx_gas_limit = currentValues.eth_tx_gas_limit
        this.erc20_tx_gas_limit = currentValues.erc20_tx_gas_limit
    },
    inject: ['privateKeyJwk', 'publicKeyDer'],
}