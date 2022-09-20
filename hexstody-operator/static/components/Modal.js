export const Modal = {
    template:
        /*html*/
        `<transition name="modal-fade">
            <div class="modal-backdrop">
                <div class="card">
                    <header>
                        <slot name="header"></slot>
                    </header>
                    <slot name="body"></slot>
                    <footer class="is-right">
                        <slot name="footer"></slot>
                        <button class="button" @click="close">
                            Close
                        </button>
                    </footer>
                </div>
            </div>
        </transition>`,
    methods: {
        close() {
            this.$emit('close')
        },
    },
}
