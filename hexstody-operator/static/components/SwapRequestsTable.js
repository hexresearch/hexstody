import {
    getSwapRequests,
    getRequiredConfirmations,
} from "../scripts/common.js"

export const SwapRequestsTable = {
    template:
        /*html*/
        `<div>
            <h4>Swap requests</h4>
            <table>
                <thead>
                    <tr>
                        <th>Time</th>
                        <th>ID</th>
                        <th>User</th>
                        <th>Pair</th>
                        <th>Amount</th>
                        <th>Status</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="swapRequest in swapRequests">
                        <td>{{swapRequest.time}}</td>
                        <td>{{swapRequest.id}}</td>
                        <td>{{swapRequest.user}}</td>
                        <td>{{swapRequest.currencyFrom}}/{{swapRequest.currencyTo}}</td>
                        <td>{{swapRequest.amountFrom}}/{{swapRequest.amountTo}}</td>
                        <td>{{swapRequest.status}}</td>
                    </tr>
                </tbody>
            </table>
        </div>`,
    data() {
        return {
            swapRequests: [],
            requiredConfirmations: null,
        }
    },
    methods: {
        async fetchData() {
            const swapRequestsResponse = await getSwapRequests(this.privateKeyJwk, this.publicKeyDer, 'all')
            // Get swap requests and sort them by date
            this.swapRequests = (await swapRequestsResponse.json()).sort(
                function (a, b) {
                    const dateA = new Date(a.created_at)
                    const dateB = new Date(b.created_at)
                    return dateB - dateA
                }
            )
            const requiredConfirmationsResponse = await getRequiredConfirmations(this.privateKeyJwk, this.publicKeyDer)
            this.requiredConfirmations = await requiredConfirmationsResponse.json()
        },
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
        }
    }
}