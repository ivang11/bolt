<template>
    <div v-show="visible" class="scroll-area">
        <div v-if="loading" class="empty-state">
            <div class="spin-ring"></div>
            <span>Connecting to Bolt...</span>
        </div>
        <div v-else-if="error" class="empty-state danger">
            {{ error }}
        </div>
        <div v-else-if="projects.length === 0" class="empty-state">
            <svg
                width="28"
                height="28"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
            >
                <rect x="3" y="3" width="7" height="7" rx="1" />
                <rect x="14" y="3" width="7" height="7" rx="1" />
                <rect x="3" y="14" width="7" height="7" rx="1" />
                <rect x="14" y="14" width="7" height="7" rx="1" />
            </svg>
            <span>No projects found</span>
        </div>
        <div v-else-if="filteredProjects.length === 0" class="empty-state">
            <svg
                width="28"
                height="28"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
            >
                <circle cx="11" cy="11" r="8" />
                <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            <span>No projects match "{{ searchQuery }}"</span>
        </div>
        <div v-else class="grid">
            <ProjectCard
                v-for="project in filteredProjects"
                :key="project.name"
                :project="project"
                :selected="drawerName === project.name"
                :busy="Boolean(busy[project.name])"
                :busy-subdir="busySubdir"
                @open="drawerStore.toggleDrawer"
                @action="projectsStore.doAction"
                @subdir-action="projectsStore.doSubdirAction"
            />
        </div>
    </div>
</template>

<script setup>
import { storeToRefs } from "pinia";
import ProjectCard from "./ProjectCard.vue";
import { useDrawerStore } from "../stores/drawer.js";
import { useProjectsStore } from "../stores/projects.js";

defineProps({
    visible: {
        type: Boolean,
        required: true,
    },
});

const projectsStore = useProjectsStore();
const drawerStore = useDrawerStore();
const {
    loading,
    error,
    projects,
    filteredProjects,
    searchQuery,
    busy,
    busySubdir,
} = storeToRefs(projectsStore);
const { drawerName } = storeToRefs(drawerStore);
</script>
