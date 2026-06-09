<template>
    <div class="dw-section">
        <div class="dw-label">Location</div>
        <div v-if="loading" class="dw-loading">
            <div class="spin-ring small"></div>
            <span>Loading...</span>
        </div>
        <div v-else class="dw-path-row">
            <span class="dw-path">{{ path || "—" }}</span>
            <button
                v-if="path"
                class="copy-btn"
                @click="$emit('copy')"
                :title="copied ? 'Copied!' : 'Copy path'"
            >
                <svg
                    v-if="!copied"
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                >
                    <rect x="9" y="9" width="13" height="13" rx="2" />
                    <path
                        d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
                    />
                </svg>
                <svg
                    v-else
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                >
                    <polyline points="20 6 9 17 4 12" />
                </svg>
            </button>
        </div>
    </div>
</template>

<script setup>
defineProps({
    path: {
        type: String,
        default: "",
    },
    loading: {
        type: Boolean,
        required: true,
    },
    copied: {
        type: Boolean,
        required: true,
    },
});

defineEmits(["copy"]);
</script>
