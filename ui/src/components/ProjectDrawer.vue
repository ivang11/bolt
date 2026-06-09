<template>
    <transition name="drawer-slide">
        <div
            v-if="drawerName"
            class="drawer"
            :style="{ width: drawerWidth + 'px' }"
        >
            <div
                class="resize-handle"
                @mousedown.prevent="drawer.startResize"
            ></div>

            <div class="drawer-head">
                <div class="drawer-title-row">
                    <span
                        class="pulse-dot"
                        :class="drawerProject?.status ?? 'stopped'"
                    ></span>
                    <span class="drawer-project-name">{{ drawerName }}</span>
                    <span
                        class="drawer-status-badge"
                        :class="drawerProject?.status"
                        >{{ drawerProject?.status ?? "—" }}</span
                    >
                </div>
                <button class="icon-btn" @click="drawer.closeDrawer">
                    <svg
                        width="12"
                        height="12"
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

            <div class="drawer-body">
                <DrawerLocation
                    :path="drawerInfo.path"
                    :loading="drawerInfoLoading"
                    :copied="copied"
                    @copy="drawer.copyPath"
                />

                <DrawerPorts
                    v-if="!drawerInfoLoading"
                    :ports="drawerInfo.ports ?? []"
                />

                <div
                    v-if="drawerProject?.subdirs?.length > 0"
                    class="dw-section"
                >
                    <div class="dw-label">Services</div>
                    <div class="dw-services">
                        <div
                            v-for="service in drawerProject.subdirs"
                            :key="service.name"
                            class="dw-service-row"
                            @click.stop
                        >
                            <button
                                v-if="service.status === 'running'"
                                class="svc-btn shell"
                                @click="openShell(drawerName, service.name)"
                                title="Shell"
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
                                    <polyline points="4 17 10 11 4 5" />
                                    <line x1="12" y1="19" x2="20" y2="19" />
                                </svg>
                            </button>
                            <span v-else class="svc-btn-placeholder"></span>

                            <span class="subdir-badge" :class="service.status">
                                <span
                                    class="subdir-dot"
                                    :class="service.status"
                                ></span>
                                {{ service.name }}
                            </span>
                            <SubdirControls
                                :project-name="drawerName"
                                :subdir="service"
                                :busy="
                                    Boolean(
                                        busySubdir[
                                            drawerName + '/' + service.name
                                        ],
                                    )
                                "
                                @action="projects.doSubdirAction"
                            />
                        </div>
                    </div>
                    <ProjectActions
                        :project="drawerProject"
                        :busy="Boolean(busy[drawerName])"
                        :building="Boolean(building[drawerName])"
                        @action="projects.doAction(drawerName, $event)"
                        @build="
                            build.doBuild(drawerName, projects.setActionError)
                        "
                    />
                </div>

                <div v-if="drawerComposeServices.length > 0" class="dw-section">
                    <div class="dw-label">Services</div>
                    <div class="dw-services">
                        <div
                            v-for="service in drawerComposeServices"
                            :key="service.name"
                            class="dw-service-row"
                            @click.stop
                        >
                            <button
                                class="svc-btn shell"
                                :disabled="service.status !== 'running'"
                                @click="openShell(drawerName, service.name)"
                                title="Shell"
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
                                    <polyline points="4 17 10 11 4 5" />
                                    <line x1="12" y1="19" x2="20" y2="19" />
                                </svg>
                            </button>
                            <span class="subdir-badge" :class="service.status">
                                <span
                                    class="subdir-dot"
                                    :class="service.status"
                                ></span>
                                {{ service.name }}
                            </span>
                        </div>
                    </div>
                    <ProjectActions
                        :project="drawerProject"
                        :busy="Boolean(busy[drawerName])"
                        :building="Boolean(building[drawerName])"
                        @action="projects.doAction(drawerName, $event)"
                        @build="
                            build.doBuild(drawerName, projects.setActionError)
                        "
                    />
                </div>

                <ShellPanel
                    :visible="shellVisible"
                    :service="shellService"
                    :set-term-ref="setTermRef"
                    @close="closeShell"
                />

                <BuildOutput
                    :logs="buildLogs"
                    :building="Boolean(building[drawerName])"
                    :set-build-ref="build.setBuildRef"
                    @clear="build.clearBuildLogs"
                />

                <LogsPanel
                    :project="drawerProject"
                    :open="logsOpen"
                    :subdir="drawerSubdir"
                    :logs="logs"
                    :set-log-ref="drawer.setLogRef"
                    @toggle="drawer.toggleLogs"
                    @switch-subdir="drawer.switchDrawerSubdir"
                />
            </div>
        </div>
    </transition>
</template>

<script setup>
import { storeToRefs } from "pinia";
import BuildOutput from "./BuildOutput.vue";
import DrawerLocation from "./DrawerLocation.vue";
import DrawerPorts from "./DrawerPorts.vue";
import LogsPanel from "./LogsPanel.vue";
import ProjectActions from "./ProjectActions.vue";
import ShellPanel from "./ShellPanel.vue";
import SubdirControls from "./SubdirControls.vue";
import { useBuildLogsStore } from "../stores/buildLogs.js";
import { useDrawerStore } from "../stores/drawer.js";
import { useProjectsStore } from "../stores/projects.js";

defineProps({
    shellVisible: {
        type: Boolean,
        required: true,
    },
    shellService: {
        type: String,
        default: null,
    },
    setTermRef: {
        type: Function,
        required: true,
    },
    openShell: {
        type: Function,
        required: true,
    },
    closeShell: {
        type: Function,
        required: true,
    },
});

const projects = useProjectsStore();
const drawer = useDrawerStore();
const build = useBuildLogsStore();
const { busy, busySubdir } = storeToRefs(projects);
const { building, buildLogs } = storeToRefs(build);
const {
    drawerName,
    drawerWidth,
    drawerProject,
    drawerInfo,
    drawerInfoLoading,
    drawerComposeServices,
    drawerSubdir,
    copied,
    logsOpen,
    logs,
} = storeToRefs(drawer);
</script>
