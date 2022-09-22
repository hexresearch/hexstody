import { Liquidity } from "./Liquidity.js"
import { SwapRequestsTable } from "./SwapRequestsTable.js"

export const SwapRequests = {
    components: {
        Liquidity,
        SwapRequestsTable
    },
    template:
        /*html*/
        `<div>
            <liquidity :private-key-jwk="privateKeyJwk" :public-key-der="publicKeyDer" />
            <swap-requests-table :private-key-jwk="privateKeyJwk" :public-key-der="publicKeyDer" />
        </div>`,
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