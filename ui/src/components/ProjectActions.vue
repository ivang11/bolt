<template>
    <div class="dw-section-actions">
        <button
            v-if="project?.status === 'stopped'"
            class="act-btn green"
            :disabled="busy"
            @click="$emit('action', 'start')"
        >
            Start
        </button>
        <button
            v-if="project?.status === 'running'"
            class="act-btn red"
            :disabled="busy"
            @click="$emit('action', 'stop')"
        >
            Stop
        </button>
        <button
            v-if="project?.status === 'running'"
            class="act-btn blue"
            :disabled="busy"
            @click="$emit('action', 'restart')"
        >
            Restart
        </button>
        <button
            class="act-btn muted"
            :disabled="busy || building"
            @click="$emit('build')"
            title="Rebuild Docker images"
        >
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
            Build
        </button>
        <span v-if="busy || building" class="busy-dots"
            ><span></span><span></span><span></span
        ></span>
    </div>
</template>

<script setup>
defineProps({
    project: {
        type: Object,
        default: null,
    },
    busy: {
        type: Boolean,
        required: true,
    },
    building: {
        type: Boolean,
        required: true,
    },
});

defineEmits(["action", "build"]);
</script>
