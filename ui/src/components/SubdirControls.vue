<template>
    <div class="subdir-btns">
        <button v-if="busy" class="svc-btn" disabled>
            <svg
                width="9"
                height="9"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                class="spinning"
            >
                <path d="M21 2v6h-6" />
                <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
                <path d="M3 22v-6h6" />
                <path d="M21 12a9 9 0 0 1-15 6.7L3 16" />
            </svg>
        </button>
        <template v-else>
            <button
                v-if="subdir.status === 'stopped'"
                class="svc-btn start"
                @click="$emit('action', projectName, subdir.name, 'start')"
                title="Start"
            >
                <svg
                    width="8"
                    height="9"
                    viewBox="0 0 10 12"
                    fill="currentColor"
                >
                    <polygon points="0,0 10,6 0,12" />
                </svg>
            </button>
            <template v-else>
                <button
                    class="svc-btn restart"
                    @click="
                        $emit('action', projectName, subdir.name, 'restart')
                    "
                    title="Restart"
                >
                    <svg
                        width="12"
                        height="12"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path d="M1 4v6h6" />
                        <path d="M3.51 15a9 9 0 1 0 .49-4.5L1 10" />
                    </svg>
                </button>
                <button
                    class="svc-btn stop"
                    @click="$emit('action', projectName, subdir.name, 'stop')"
                    title="Stop"
                >
                    <svg
                        width="8"
                        height="8"
                        viewBox="0 0 10 10"
                        fill="currentColor"
                    >
                        <rect width="10" height="10" rx="1.5" />
                    </svg>
                </button>
            </template>
        </template>
    </div>
</template>

<script setup>
defineProps({
    projectName: {
        type: String,
        required: true,
    },
    subdir: {
        type: Object,
        required: true,
    },
    busy: {
        type: Boolean,
        default: false,
    },
});

defineEmits(["action"]);
</script>
