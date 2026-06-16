<template>
    <div class="app">
        <TopBar />

        <ErrorBanner :message="actionError" @dismiss="actionError = null" />

        <div class="workspace">
            <div class="main">
                <ProjectsView :visible="view === 'projects'" />

                <ConfigView :visible="view === 'config'" />
            </div>

            <ProjectDrawer
                :shell-visible="shellVisible"
                :shell-service="shellService"
                :set-term-ref="setTermRef"
                :open-shell="openProjectShell"
                :close-shell="closeShell"
            />
        </div>
    </div>
</template>

<script setup>
import { storeToRefs } from "pinia";
import { onUnmounted } from "vue";
import ConfigView from "./components/ConfigView.vue";
import ErrorBanner from "./components/ErrorBanner.vue";
import ProjectDrawer from "./components/ProjectDrawer.vue";
import ProjectsView from "./components/ProjectsView.vue";
import TopBar from "./components/TopBar.vue";
import { useEscapeKey } from "./composables/useEscapeKey.js";
import { useProjectPolling } from "./composables/useProjectPolling.js";
import { useProjectShellSession } from "./composables/useProjectShellSession.js";
import { useShell } from "./composables/useShell.js";
import { useBuildLogsStore } from "./stores/buildLogs.js";
import { useDrawerStore } from "./stores/drawer.js";
import { useProjectsStore } from "./stores/projects.js";
import { useUiStore } from "./stores/ui.js";

const uiStore = useUiStore();
const projectsStore = useProjectsStore();
const buildStore = useBuildLogsStore();
const drawerStore = useDrawerStore();
const { view } = storeToRefs(uiStore);
const { actionError } = storeToRefs(projectsStore);
const { drawerName } = storeToRefs(drawerStore);
const { fetchProjects } = projectsStore;
const { closeBuildSource } = buildStore;
const { stopLogs, fetchDrawerInfo, closeDrawer } = drawerStore;

projectsStore.setAfterProjectAction((name) => {
    if (drawerName.value === name) fetchDrawerInfo(name);
});

const shell = useShell();
const {
    shellProject,
    shellService,
    closeShell,
    shellVisible,
    setTermRef,
    fitShell,
    confirmShellLoss,
    openProjectShell,
} = useProjectShellSession({
    getDrawerName: () => drawerName.value,
    shell,
});

drawerStore.configureShell({
    fitShell,
    closeShell,
    shellProject,
    shellService,
    confirmShellLoss,
});

useEscapeKey(() => {
    if (drawerName.value) closeDrawer();
});
useProjectPolling(fetchProjects);

onUnmounted(() => {
    stopLogs();
    closeBuildSource();
});
</script>
