import { WithdrawalRequests } from "./WithdrawalRequests.js"
import { Invites } from "./Invites.js"
import { WithdrawalLimits } from "./WithdrawalLimits.js"
import { ExchangeRequests } from "./ExchangeRequests.js"

export const AuthorizedContent = {
    components: {
        WithdrawalRequests, Invites, WithdrawalLimits, ExchangeRequests
    },
    template:
        /*html*/
        `<div class="card">
            <header>
                <nav class="tabs is-left">
                    <a v-for="tab in tabs" :key="tab" :class="{ active: currentTab === tab }" @click="currentTab = tab" href="javascript:void(0)" role="button">
                        {{ getTabName(tab) }}
                    </a>
                </nav>
            </header>
            <KeepAlive>
                <component :is="currentTab" :private-key-jwk="privateKeyJwk" :public-key-der="publicKeyDer"></component>
            </KeepAlive>
        </div>`,
    data() {
        return {
            currentTab: 'WithdrawalRequests',
            tabs: ['WithdrawalRequests', 'Invites', 'WithdrawalLimits', 'ExchangeRequests']
        }
    },
    methods: {
        getTabName(tab) {
            let tabName
            switch (tab) {
                case 'WithdrawalRequests':
                    tabName = 'Withdrawal requests'
                    break
                case 'Invites':
                    tabName = 'Invites'
                    break
                case 'WithdrawalLimits':
                    tabName = 'Withdrawal limits'
                    break
                case 'ExchangeRequests':
                    tabName = 'Exchange requests'
                    break
                default:
                    tabName = 'Undefined'
            };
            return tabName
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
