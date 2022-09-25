import {
    getWithdrawalRequests,
    getRequiredConfirmations,
    confirmWithdrawalRequest,
    rejectWithdrawalRequest,
    getUserInfo,
    copyToClipboard,
    getCurrencyName,
    formatCurrencyValue,
    formatAddress,
    formatWithdrawalRequestStatus,
    formatExplorerLink,
    truncate,
    truncateMiddle,
} from "../scripts/common.js"

import { Modal } from "./Modal.js"

export const WithdrawalRequestsTable = {
    components: {
        Modal
    },
    template:
        /*html*/
        `<div>
            <h4>Withdrawal requests</h4>
            <div class="table-container">
                <table>
                    <thead>
                        <tr>
                            <th>Time</th>
                            <th>ID</th>
                            <th>User</th>
                            <th>Withdrawal address</th>
                            <th>Amount, {{getCurrencyName(currency)}}</th>
                            <th>Status</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="withdrawalRequest in withdrawalRequests">
                            <td>{{withdrawalRequest.created_at}}</td>
                            <td>
                                <div class="flex-row">
                                    <span v-tippy="withdrawalRequest.id">
                                        {{truncate(withdrawalRequest.id, 8)}}
                                    </span>
                                    <button class="button clear icon-only" @click='copyToClipboard(withdrawalRequest.id)' v-tippy>
                                        <span class="mdi mdi-content-copy"></span>
                                    </button>
                                    <tippy trigger="click" :hide-on-click="false" @show="hideTooltip">
                                        Copied
                                    </tippy>
                                </div>
                            </td>
                            <td>{{withdrawalRequest.user}}</td>
                            <td>
                                <div class="flex-row">
                                    <span v-tippy="formatAddress(withdrawalRequest.address)">
                                        {{truncateMiddle((formatAddress(withdrawalRequest.address)), 15)}}
                                    </span>
                                    <button class="button clear icon-only" @click='copyToClipboard(formatAddress(withdrawalRequest.address))' v-tippy>
                                        <span class="mdi mdi-content-copy"></span>
                                    </button>
                                    <tippy trigger="click" :hide-on-click="false" @show="hideTooltip">
                                        Copied
                                    </tippy>
                                </div>
                            </td>
                            <td>{{formatCurrencyValue(currency, withdrawalRequest.amount)}}</td>
                            <td>
                                <div class="flex-row">
                                    {{formatWithdrawalRequestStatus(withdrawalRequest.confirmation_status, requiredConfirmations)}}
                                    <a v-if="withdrawalRequest.confirmation_status.type === 'Completed'" class="button clear icon-only" :href="formatExplorerLink(withdrawalRequest.confirmation_status.txid)" v-tippy="{content: 'Block explorer link'}">
                                        <span class="mdi mdi-link"></span>
                                    </a>
                                </div>
                            </td>
                            <td>
                                <div class="action-buttons-wrapper justify-center">
                                    <button class="button primary" @click="confirmRequest(withdrawalRequest)" :disabled="withdrawalRequest.confirmation_status.type !== 'InProgress'">Confirm</button>
                                    <button class="button error" @click="rejectRequest(withdrawalRequest)" :disabled="withdrawalRequest.confirmation_status.type !== 'InProgress'">Reject</button>
                                    <button class="button" @click="showRequestDetails(withdrawalRequest)">Details</button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <Modal v-show="isModalVisible" @close="closeModal">
                <template v-slot:header>
                    <h4>Withdrawal request details</h4>
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
    methods: {
        formatCurrencyValue,
        truncate,
        truncateMiddle,
        formatAddress,
        formatWithdrawalRequestStatus,
        formatExplorerLink,
        getCurrencyName,
        copyToClipboard,
        async fetchData() {
            const withdrawalRequestsResponse = await getWithdrawalRequests(this.privateKeyJwk, this.publicKeyDer, this.currency, this.filter)
            // Get withdrawal requests and sort them by date
            this.withdrawalRequests = (await withdrawalRequestsResponse.json()).sort(
                function (a, b) {
                    const dateA = new Date(a.created_at)
                    const dateB = new Date(b.created_at)
                    return dateB - dateA
                }
            )
            const requiredConfirmationsResponse = await getRequiredConfirmations(this.privateKeyJwk, this.publicKeyDer)
            this.requiredConfirmations = await requiredConfirmationsResponse.json()
        },
        hideTooltip(instance) {
            setTimeout(() => {
                instance.hide()
            }, 1000)
        },
        confirmRequest(withdrawalRequest) {
            // Here we copy withdrawalRequest and remove confirmation status feild
            // Note that the order of the fields affects the signature verification
            let confirmationData = (({ confirmation_status, ...o }) => o)(withdrawalRequest)
            confirmWithdrawalRequest(this.privateKeyJwk, this.publicKeyDer, confirmationData)
            this.fetchData()
        },
        rejectRequest(withdrawalRequest) {
            // Here we copy withdrawalRequest and remove confirmation status feild
            // Note that the order of the fields affects the signature verification
            let confirmationData = (({ confirmation_status, ...o }) => o)(withdrawalRequest)
            rejectWithdrawalRequest(this.privateKeyJwk, this.publicKeyDer, confirmationData)
            this.fetchData()
        },
        async showRequestDetails(withdrawalRequest) {
            const userInfoResponse = await getUserInfo(this.privateKeyJwk, this.publicKeyDer, withdrawalRequest.user)
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
    data() {
        return {
            withdrawalRequests: [],
            requiredConfirmations: null,
            isModalVisible: false,
            userInfo: null,
            filter: "pending",
        }
    },
    watch: {
        currency: 'fetchData'
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
    },
}