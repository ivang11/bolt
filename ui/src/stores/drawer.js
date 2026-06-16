import { defineStore } from "pinia";
import { computed, nextTick, ref, watch } from "vue";
import { apiSegment } from "../composables/api.js";
import { useBuildLogsStore } from "./buildLogs.js";
import { useProjectsStore } from "./projects.js";

const MIN_DRAWER_WIDTH = 260;
const MAX_DRAWER_WIDTH = 1400;

export const useDrawerStore = defineStore("drawer", () => {
    const projectsStore = useProjectsStore();
    const buildStore = useBuildLogsStore();

    const drawerName = ref(null);
    const drawerInfo = ref({ path: "", ports: [] });
    const drawerInfoLoading = ref(false);
    const drawerComposeServices = ref([]);
    const drawerSubdir = ref(null);
    const drawerWidth = ref(900);
    const logsOpen = ref(false);
    const copied = ref(false);
    const logs = ref([]);
    const logBody = ref(null);
    let eventSource = null;
    let shellIntegration = {
        fitShell: () => {},
        closeShell: () => {},
        shellProject: null,
        shellService: null,
        confirmShellLoss: () => true,
    };

    const drawerProject = computed(
        () =>
            projectsStore.projects.find((p) => p.name === drawerName.value) ??
            null,
    );

    function configureShell(integration) {
        shellIntegration = { ...shellIntegration, ...integration };
    }

    function setLogRef(el) {
        logBody.value = el;
    }

    function connectLogs(name, subdir) {
        if (eventSource) {
            eventSource.close();
            eventSource = null;
        }
        logs.value = [];
        const url = subdir
            ? `/api/projects/${apiSegment(name)}/logs?subdir=${apiSegment(subdir)}`
            : `/api/projects/${apiSegment(name)}/logs`;
        eventSource = new EventSource(url);
        eventSource.onmessage = (e) => {
            logs.value = [...logs.value, e.data];
            if (logs.value.length > 1000) logs.value = logs.value.slice(-900);
            nextTick(() => {
                if (logBody.value) {
                    logBody.value.scrollTop = logBody.value.scrollHeight;
                }
            });
        };
        eventSource.onerror = () => {
            if (eventSource?.readyState === EventSource.CLOSED) {
                eventSource = null;
                logsOpen.value = false;
            }
        };
    }

    function stopLogs() {
        if (eventSource) {
            eventSource.close();
            eventSource = null;
        }
        logs.value = [];
    }

    async function fetchDrawerInfo(name) {
        const requestedName = name;
        drawerInfoLoading.value = true;
        drawerInfo.value = { path: "", ports: [] };
        try {
            const res = await fetch(`/api/projects/${apiSegment(name)}/info`);
            const data = await res.json();
            if (drawerName.value !== requestedName) return;
            drawerInfo.value = {
                path: data.path ?? "",
                ports: data.ports ?? [],
            };
        } finally {
            if (drawerName.value === requestedName) {
                drawerInfoLoading.value = false;
            }
        }
    }

    async function fetchComposeServices(name) {
        const requestedName = name;
        const res = await fetch(`/api/projects/${apiSegment(name)}/services`);
        const data = await res.json();
        if (drawerName.value !== requestedName) return;
        drawerComposeServices.value = data.services ?? [];
    }

    function toggleDrawer(name) {
        if (drawerName.value === name) {
            closeDrawer();
            return;
        }
        stopLogs();
        drawerName.value = name;
        drawerSubdir.value = null;
        drawerComposeServices.value = [];
        logsOpen.value = false;
        fetchDrawerInfo(name);
        const proj = projectsStore.projects.find((p) => p.name === name);
        if (!proj?.subdirs?.length) fetchComposeServices(name);
    }

    function toggleLogs() {
        if (!logsOpen.value) {
            logsOpen.value = true;
            const first = drawerProject.value?.subdirs?.[0]?.name ?? null;
            drawerSubdir.value = first;
            connectLogs(drawerName.value, first);
        } else {
            logsOpen.value = false;
            stopLogs();
        }
    }

    function closeDrawer() {
        const shellProject = shellIntegration.shellProject?.value;
        const shellService = shellIntegration.shellService?.value;
        const drawerShellOpen =
            shellService && shellProject === drawerName.value;
        if (drawerShellOpen && !shellIntegration.confirmShellLoss()) return;
        drawerName.value = null;
        drawerSubdir.value = null;
        drawerInfo.value = { path: "", ports: [] };
        drawerComposeServices.value = [];
        logsOpen.value = false;
        buildStore.clearBuildLogs();
        buildStore.closeBuildSource();
        stopLogs();
        if (drawerShellOpen) shellIntegration.closeShell();
    }

    function switchDrawerSubdir(subdir) {
        drawerSubdir.value = subdir;
        if (logsOpen.value) connectLogs(drawerName.value, subdir);
    }

    async function copyPath() {
        if (!drawerInfo.value.path) return;
        await navigator.clipboard.writeText(drawerInfo.value.path);
        copied.value = true;
        setTimeout(() => {
            copied.value = false;
        }, 2000);
    }

    function startResize() {
        const onMove = (event) => {
            const newWidth = window.innerWidth - event.clientX;
            drawerWidth.value = Math.max(
                MIN_DRAWER_WIDTH,
                Math.min(MAX_DRAWER_WIDTH, newWidth),
            );
            nextTick(() => shellIntegration.fitShell());
        };
        const onUp = () => {
            document.removeEventListener("mousemove", onMove);
            document.removeEventListener("mouseup", onUp);
            document.body.style.cursor = "";
            document.body.style.userSelect = "";
        };
        document.body.style.cursor = "col-resize";
        document.body.style.userSelect = "none";
        document.addEventListener("mousemove", onMove);
        document.addEventListener("mouseup", onUp);
    }

    watch(
        () => projectsStore.projects,
        (updated) => {
            if (!drawerName.value) return;
            const proj = updated.find((p) => p.name === drawerName.value);
            if (proj && proj.status !== "running" && eventSource) {
                stopLogs();
                logsOpen.value = false;
            }
        },
    );

    return {
        drawerName,
        drawerInfo,
        drawerInfoLoading,
        drawerComposeServices,
        drawerSubdir,
        drawerWidth,
        logsOpen,
        copied,
        logs,
        drawerProject,
        configureShell,
        setLogRef,
        stopLogs,
        fetchDrawerInfo,
        toggleDrawer,
        toggleLogs,
        closeDrawer,
        switchDrawerSubdir,
        copyPath,
        startResize,
    };
});
