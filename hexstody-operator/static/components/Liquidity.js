export const Liquidity = {
    template:
        /*html*/
        `<div>
            Liquidity
        </div>`,
    data() {
        return {
        }
    },
    methods: {
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