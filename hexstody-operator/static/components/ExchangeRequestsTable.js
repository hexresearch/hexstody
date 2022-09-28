import {
    getExchangeRequests,
    getRequiredConfirmations,
    confirmExchangeRequest,
    rejectExchangeRequest,
    copyToClipboard,
    truncate
} from "../scripts/common.js"

import { Modal } from "./Modal.js"

export const ExchangeRequestsTable = {
    components: {
        Modal
    },
    template:
        /*html*/
        `<div>
            <h4>Exchange requests</h4>
            <div class="table-container">
                <table>
                    <thead>
                        <tr>
                            <th>Time</th>
                            <th>ID</th>
                            <th>User</th>
                            <th>Pair</th>
                            <th>Amount</th>
                            <th>Status</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="exchangeRequest in exchangeRequests">
                            <td>{{exchangeRequest.created_at}}</td>
                            <td>
                                <div class="flex-row">
                                    <span v-tippy="exchangeRequest.id">
                                        {{truncate(exchangeRequest.id, 8)}}
                                    </span>
                                    <button class="button clear icon-only" @click='copyToClipboard(exchangeRequest.id)' v-tippy>
                                        <span class="mdi mdi-content-copy"></span>
                                    </button>
                                    <tippy trigger="click" :hide-on-click="false" @show="hideTooltip">
                                        Copied
                                    </tippy>
                                </div>
                            </td>
                            <td>{{exchangeRequest.user}}</td>
                            <td>{{exchangeRequest.currency_from}}/{{exchangeRequest.currency_to}}</td>
                            <td>{{exchangeRequest.amount_from}}/{{exchangeRequest.amount_to}}</td>
                            <td>{{exchangeRequest.status}}</td>
                            <td>
                                <div class="action-buttons-wrapper justify-center">
                                    <button class="button primary" @click="confirmRequest(exchangeRequest)" :disabled="exchangeRequest.status.type !== 'InProgress'">Confirm</button>
                                    <button class="button error" @click="rejectRequest(exchangeRequest)" :disabled="exchangeRequest.status.type !== 'InProgress'">Reject</button>
                                    <button class="button" @click="showRequestDetails(exchangeRequest)">Details</button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <Modal v-show="isModalVisible" @close="closeModal">
                <template v-slot:header>
                    <h4>Exchange request details</h4>
                </template>
                <template v-slot:body v-if="userInfo">
                    <p><b>First name:</b> {{userInfo.firstName}}</p>
                    <p><b>Last name:</b> {{userInfo.lastName}}</p>
                    <p><b>Email:</b> {{userInfo.email ? userInfo.email.email : ""}}</p>
                    <p><b>Phone:</b> {{userInfo.phone ? userInfo.phone.number : ""}}</p>
                    <p><b>Telegram:</b> {{userInfo.tgName}}</p>
                </template>
                <template v-slot:footer>
                </template>
            </Modal>
        </div>`,
    data() {
        return {
            exchangeRequests: [],
            requiredConfirmations: null,
            isModalVisible: false,
            userInfo: null,
            filter: "all"
        }
    },
    methods: {
        copyToClipboard,
        truncate,
        async fetchData() {
            const exchangeRequestsResponse = await getExchangeRequests(this.privateKeyJwk, this.publicKeyDer, this.filter)
            // Get exchange requests and sort them by date
            this.exchangeRequests = (await exchangeRequestsResponse.json()).sort(
                function (a, b) {
                    const dateA = new Date(a.created_at)
                    const dateB = new Date(b.created_at)
                    return dateB - dateA
                }
            )
            const requiredConfirmationsResponse = await getRequiredConfirmations(this.privateKeyJwk, this.publicKeyDer)
            this.requiredConfirmations = await requiredConfirmationsResponse.json()
        },
        confirmRequest(exchangeRequest) {
            // Here we copy exchangeRequest and remove status feild
            // Note that the order of the fields affects the signature verification
            let confirmationData = (({ status, ...o }) => o)(exchangeRequest)
            confirmExchangeRequest(this.privateKeyJwk, this.publicKeyDer, confirmationData)
            this.fetchData()
        },
        rejectRequest(exchangeRequest) {
            // Here we copy exchangeRequest and remove status feild
            // Note that the order of the fields affects the signature verification
            let confirmationData = (({ status, ...o }) => o)(exchangeRequest)
            rejectExchangeRequest(this.privateKeyJwk, this.publicKeyDer, confirmationData)
            this.fetchData()
        },
        async showRequestDetails(exchangeRequest) {
            const userInfoResponse = await getUserInfo(this.privateKeyJwk, this.publicKeyDer, exchangeRequest.user)
            let userInfo = await userInfoResponse.json()
            this.userInfo = userInfo
            this.showModal()
        },
        showModal() {
            this.isModalVisible = true
        },
        closeModal() {
            this.isModalVisible = false
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
        currency: {}
    }
}