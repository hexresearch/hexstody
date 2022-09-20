import { getInvite, getInvites, copyToClipboard } from "../scripts/common.js"

export const Invites = {
    template:
        /*html*/
        `<div class="flex-column">
            <div>
                <label for="invite-input">Invite label</label>
                <input type="text" id="invite-input" :class="{ 'error': hasError }" v-model="label">
                <div v-if="hasError" class="text-error">{{errorMessage}}</div>
            </div>
            <div class="action-buttons-wrapper">
                <button class="button" @click='generateInvite'>Generate new invite</button>
                <button class="button" @click='toggleInvites'>{{showInvites ? 'Hide invites' : 'Show invites'}}</button>
            </div>
            <div v-if='invite'>
                <h4>Generated invite</h4>
                <div class="table-container">
                    <table>
                        <thead>
                            <tr>
                                <th>Label</th>
                                <th>Invite</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>{{invite.label}}</td>
                                <td>
                                    <div class="flex-row">
                                        {{invite.invite.invite}}
                                        <tippy trigger="click" :hide-on-click="false" @show="hideTooltip">
                                            Copied
                                        </tippy>
                                        <button class="button clear icon-only" @click='copyToClipboard(invite.invite.invite)' v-tippy>
                                            <span class="mdi mdi-content-copy"></span>
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
            <div v-if='showInvites'>
                <h4>Previously generated invites</h4>
                <div class="table-container">
                    <table>
                        <thead>
                            <tr>
                                <th>Label</th>
                                <th>Invite</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr v-for="invite in invites" :key="invite">
                                <td>{{invite.label}}:</td>
                                <td>
                                    <div class="flex-row">
                                        {{invite.invite.invite}}
                                        <tippy trigger="click" :hide-on-click="false" @show="hideTooltip">
                                            Copied
                                        </tippy>
                                        <button class="button clear icon-only" @click='copyToClipboard(invite.invite.invite)' v-tippy>
                                            <span class="mdi mdi-content-copy"></span>
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>`,
    data() {
        return {
            hasError: false,
            errorMessage: "",
            showInvites: false,
            invite: null,
            label: "",
            invites: []
        }
    },
    methods: {
        copyToClipboard,
        async generateInvite() {
            this.invite = null
            this.hasError = false
            this.errorMessage = ""
            if (this.label) {
                const body = { label: this.label }
                const invite = await getInvite(this.privateKeyJwk, this.publicKeyDer, body)
                if (invite.ok) {
                    const inviteText = await invite.json()
                    this.invite = inviteText
                    this.hasError = false
                    this.fetchInvites()
                } else {
                    this.hasError = true
                    this.errorMessage = `Failed to generate an invite`
                }
            } else {
                this.hasError = true
                this.errorMessage = "Label field is required"
            }
        },
        async fetchInvites() {
            const response = await getInvites(this.privateKeyJwk, this.publicKeyDer)
            this.invites = await response.json()
        },
        hideTooltip(instance) {
            setTimeout(() => {
                instance.hide()
            }, 1000)
        },
        toggleInvites() {
            if (!this.showInvites) {
                this.fetchInvites()
            }
            this.showInvites = !this.showInvites
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