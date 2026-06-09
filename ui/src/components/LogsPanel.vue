<template>
    <div
        v-if="project?.status === 'running'"
        class="dw-logs"
        :class="{ open: open }"
    >
        <div class="dw-logs-header" @click="$emit('toggle')">
            <span class="dw-label" style="margin-bottom: 0">Logs</span>
            <div class="log-tabs" v-if="open" @click.stop>
                <button
                    v-for="service in project.subdirs"
                    :key="service.name"
                    class="log-tab"
                    :class="{ active: subdir === service.name }"
                    @click="$emit('switch-subdir', service.name)"
                >
                    {{ service.name }}
                </button>
                <span v-if="!project.subdirs?.length" class="log-tab active"
                    >all</span
                >
            </div>
            <svg
                class="logs-chevron"
                :class="{ rotated: open }"
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
            >
                <polyline points="6 9 12 15 18 9" />
            </svg>
        </div>
        <div v-if="open" class="dw-log-body" :ref="setLogRef">
            <div v-if="logs.length === 0" class="log-empty">
                Waiting for logs...
            </div>
            <div v-for="(line, index) in logs" :key="index" class="log-line">
                {{ line }}
            </div>
        </div>
    </div>
</template>

<script setup>
defineProps({
    project: {
        type: Object,
        default: null,
    },
    open: {
        type: Boolean,
        required: true,
    },
    subdir: {
        type: String,
        default: null,
    },
    logs: {
        type: Array,
        required: true,
    },
    setLogRef: {
        type: Function,
        required: true,
    },
});

defineEmits(["toggle", "switch-subdir"]);
</script>
