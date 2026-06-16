<template>
    <transition name="slide-down">
        <div
            v-if="message"
            class="banner error-banner"
            @click="$emit('dismiss')"
        >
            <svg
                width="13"
                height="13"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
            >
                <circle cx="12" cy="12" r="10" />
                <line x1="12" y1="8" x2="12" y2="12" />
                <line x1="12" y1="16" x2="12.01" y2="16" />
            </svg>
            <span>{{ message }}</span>
            <button class="banner-x">&#10005;</button>
        </div>
    </transition>
</template>

<script setup>
defineProps({
    message: {
        type: String,
        default: null,
    },
});

defineEmits(["dismiss"]);
</script>
