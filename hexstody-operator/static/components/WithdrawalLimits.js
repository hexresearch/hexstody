import {
    truncate,
    getLimitRequests,
    getRequiredConfirmations,
    formatTime,
    formatLimitValue,
    formatLimitStatus,
    copyToClipboard,
    getCurrencyName,
    confirmLimitRequest,
    rejectLimitRequest,
} from "../scripts/common.js"

export const WithdrawalLimits = {
    template:
        /*html*/
        `<div>
            <h4>Withdrawal limits</h4>
            <div class="table-container">
                <table>
                    <thead>
                        <tr>
                            <th>Time</th>
                            <th>ID</th>
                            <th>User</th>
                            <th>Currency</th>
                            <th>Current limit value</th>
                            <th>New limit value</th>
                            <th>Status</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="limitRequest in limitRequests">
                            <td>{{formatTime(limitRequest.created_at)}}</td>
                            <td>
                                <div class="flex-row">
                                    <span v-tippy="limitRequest.id">
                                        {{truncate(limitRequest.id, 8)}}
                                    </span>
                                    <button class="button clear icon-only" @click='copyToClipboard(limitRequest.id)' v-tippy>
                                        <span class="mdi mdi-content-copy"></span>
                                    </button>
                                    <tippy trigger="click" :hide-on-click="false" @show="hideTooltip">
                                        Copied
                                    </tippy>
                                </div>
                            </td>
                            <td>{{limitRequest.user}}</td>
                            <td>{{getCurrencyName(limitRequest.currency)}}</td>
                            <td>{{formatLimitValue(limitRequest.current_limit)}}</td>
                            <td>{{formatLimitValue(limitRequest.requested_limit)}}</td>
                            <td>{{formatLimitStatus(limitRequest.status, requiredConfirmations)}}</td>
                            <td>
                                <div class="action-buttons-wrapper justify-center">
                                    <button class="button primary" @click="confirmRequest(limitRequest)" :disabled="limitRequest.status.type !== 'InProgress'">Confirm</button>
                                    <button class="button error" @click="rejectRequest(limitRequest)" :disabled="limitRequest.status.type !== 'InProgress'">Reject</button>
                                    <!-- <button class="button" @click="showRequestDetails(withdrawalRequest)">Details</button> -->
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>`,
    methods: {
        truncate,
        formatLimitValue,
        formatLimitStatus,
        copyToClipboard,
        getCurrencyName,
        confirmLimitRequest,
        rejectLimitRequest,
        formatTime,
        async fetchData() {
            const limitRequestsResponse = await getLimitRequests(this.privateKeyJwk, this.publicKeyDer, this.filter)
            // Get limit requests and sort them by date
            this.limitRequests = (await limitRequestsResponse.json()).sort(
                function (a, b) {
                    const dateA = new Date(a.created_at)
                    const dateB = new Date(b.created_at)
                    return dateB - dateA
                }
            )
            const requiredConfirmationsResponse = await getRequiredConfirmations(this.privateKeyJwk, this.publicKeyDer)
            this.requiredConfirmations = (await requiredConfirmationsResponse.json()).change_limit
        },
        hideTooltip(instance) {
            setTimeout(() => {
                instance.hide()
            }, 1000)
        },

        confirmRequest(limitRequest) {
            // Here we copy limitRequest and remove 'status' and 'current_limit' feilds 
            // Note that the order of the fields affects the signature verification
            let confirmationData = (({ status, current_limit, ...o }) => o)(limitRequest)
            confirmLimitRequest(this.privateKeyJwk, this.publicKeyDer, confirmationData)
            this.fetchData()
        },
        rejectRequest(limitRequest) {
            // Here we copy limitRequest and remove 'status' and 'current_limit' feilds
            // Note that the order of the fields affects the signature verification
            let confirmationData = (({ status, current_limit, ...o }) => o)(limitRequest)
            rejectLimitRequest(this.privateKeyJwk, this.publicKeyDer, confirmationData)
            this.fetchData()
        },
        showRequestDetails(limitRequest) {
            // show additional info about user and request
        },
    },
    async created() {
        this.fetchData()
    },
    watch: {
        eventToggle: 'fetchData'
    },
    data() {
        return {
            limitRequests: [],
            requiredConfirmations: null,
            filter: "all"
        }
    },
    inject: ['eventToggle', 'privateKeyJwk', 'publicKeyDer'],
}