<template>
    <div v-if="logs.length > 0" class="dw-build">
        <div class="dw-build-header">
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
                    :class="{ spinning: building }"
                >
                    <path d="M21 2v6h-6" />
                    <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
                    <path d="M3 22v-6h6" />
                    <path d="M21 12a9 9 0 0 1-15 6.7L3 16" />
                </svg>
                Build{{ building ? "ing..." : " output" }}
            </span>
            <button
                class="icon-btn"
                @click="$emit('clear')"
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
        <div class="dw-log-body dw-build-body" :ref="setBuildRef">
            <div v-for="(line, index) in logs" :key="index" class="log-line">
                {{ line }}
            </div>
        </div>
    </div>
</template>

<script setup>
defineProps({
    logs: {
        type: Array,
        required: true,
    },
    building: {
        type: Boolean,
        required: true,
    },
    setBuildRef: {
        type: Function,
        required: true,
    },
});

defineEmits(["clear"]);
</script>
