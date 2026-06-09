import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { apiSegment } from "../composables/api.js";

export const useProjectsStore = defineStore("projects", () => {
    const projects = ref([]);
    const loading = ref(true);
    const error = ref(null);
    const busy = ref({});
    const busySubdir = ref({});
    const stoppingAll = ref(false);
    const searchQuery = ref("");
    const actionError = ref(null);
    let afterProjectAction = null;

    const filteredProjects = computed(() => {
        const q = searchQuery.value.trim().toLowerCase();
        if (!q) return projects.value;
        return projects.value.filter((p) => p.name.toLowerCase().includes(q));
    });

    function setActionError(message) {
        actionError.value = message;
    }

    function setAfterProjectAction(callback) {
        afterProjectAction = callback;
    }

    async function fetchProjects({ silent = false } = {}) {
        if (!silent) loading.value = true;
        error.value = null;
        try {
            const res = await fetch("/api/projects");
            const data = await res.json();
            projects.value = data.projects ?? [];
        } catch {
            error.value = 'Cannot reach Bolt API — is "bolt ui" running?';
        } finally {
            if (!silent) loading.value = false;
        }
    }

    async function doSubdirAction(project, subdir, action) {
        const key = `${project}/${subdir}`;
        busySubdir.value = { ...busySubdir.value, [key]: true };
        actionError.value = null;
        try {
            const res = await fetch(
                `/api/projects/${apiSegment(project)}/subdirs/${apiSegment(subdir)}/${action}`,
                { method: "POST" },
            );
            const data = await res.json();
            if (!data.ok) actionError.value = `${subdir}: ${data.error}`;
            await fetchProjects({ silent: true });
        } finally {
            busySubdir.value = { ...busySubdir.value, [key]: false };
        }
    }

    async function doAction(name, action) {
        busy.value = { ...busy.value, [name]: true };
        actionError.value = null;
        try {
            const res = await fetch(
                `/api/projects/${apiSegment(name)}/${action}`,
                {
                    method: "POST",
                },
            );
            const data = await res.json();
            if (data && !data.ok) actionError.value = data.error;
            await fetchProjects({ silent: true });
            afterProjectAction?.(name);
        } finally {
            busy.value = { ...busy.value, [name]: false };
        }
    }

    async function stopAllProjects() {
        stoppingAll.value = true;
        actionError.value = null;
        try {
            const res = await fetch("/api/projects/stop-all", {
                method: "POST",
            });
            const data = await res.json();
            if (data && !data.ok) actionError.value = data.error;
            await fetchProjects({ silent: true });
        } catch {
            actionError.value = "Failed to stop all projects";
        } finally {
            stoppingAll.value = false;
        }
    }

    return {
        projects,
        loading,
        error,
        busy,
        busySubdir,
        stoppingAll,
        searchQuery,
        actionError,
        filteredProjects,
        setActionError,
        setAfterProjectAction,
        fetchProjects,
        doSubdirAction,
        doAction,
        stopAllProjects,
    };
});
