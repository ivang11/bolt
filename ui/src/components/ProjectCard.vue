<template>
    <div
        class="card"
        :class="{
            running: project.status === 'running',
            selected,
        }"
        @click="$emit('open', project.name)"
    >
        <div class="card-top">
            <span class="pulse-dot" :class="project.status"></span>
            <span class="card-name">{{ project.name }}</span>
            <span class="card-status">{{ project.status }}</span>
        </div>

        <div v-if="project.subdirs.length > 0" class="card-subdirs" @click.stop>
            <div
                v-for="subdir in project.subdirs"
                :key="subdir.name"
                class="subdir-row"
            >
                <span class="subdir-badge" :class="subdir.status">
                    <span class="subdir-dot" :class="subdir.status"></span>
                    {{ subdir.name }}
                </span>
                <SubdirControls
                    :project-name="project.name"
                    :subdir="subdir"
                    :busy="
                        Boolean(busySubdir[project.name + '/' + subdir.name])
                    "
                    @action="(...args) => $emit('subdir-action', ...args)"
                />
            </div>
        </div>

        <div class="card-actions" @click.stop>
            <button
                v-if="project.status === 'stopped'"
                class="act-btn green"
                :disabled="busy"
                @click="$emit('action', project.name, 'start')"
            >
                Start
            </button>
            <button
                v-if="project.status === 'running'"
                class="act-btn red"
                :disabled="busy"
                @click="$emit('action', project.name, 'stop')"
            >
                Stop
            </button>
            <button
                class="act-btn blue"
                :disabled="busy"
                @click="$emit('action', project.name, 'restart')"
            >
                Restart
            </button>
            <span v-if="busy" class="busy-dots"
                ><span></span><span></span><span></span
            ></span>
        </div>
    </div>
</template>

<script setup>
import SubdirControls from "./SubdirControls.vue";

defineProps({
    project: {
        type: Object,
        required: true,
    },
    selected: {
        type: Boolean,
        default: false,
    },
    busy: {
        type: Boolean,
        default: false,
    },
    busySubdir: {
        type: Object,
        required: true,
    },
});

defineEmits(["open", "action", "subdir-action"]);
</script>
