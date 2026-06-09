<template>
    <div v-if="visible" class="dw-shell">
        <div class="dw-shell-header">
            <span class="dw-label" style="margin-bottom: 0">
                <svg
                    width="11"
                    height="11"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <polyline points="4 17 10 11 4 5" />
                    <line x1="12" y1="19" x2="20" y2="19" />
                </svg>
                {{ service }}
            </span>
            <button
                class="icon-btn"
                @click="$emit('close')"
                style="width: 22px; height: 22px"
            >
                <svg
                    width="10"
                    height="10"
                    viewBox="0 0 12 12"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                >
                    <line x1="1" y1="1" x2="11" y2="11" />
                    <line x1="11" y1="1" x2="1" y2="11" />
                </svg>
            </button>
        </div>
        <div :ref="setTermRef" class="dw-shell-term"></div>
    </div>
</template>

<script setup>
defineProps({
    visible: {
        type: Boolean,
        required: true,
    },
    service: {
        type: String,
        default: null,
    },
    setTermRef: {
        type: Function,
        required: true,
    },
});

defineEmits(["close"]);
</script>
