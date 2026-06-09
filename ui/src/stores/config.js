import { defineStore } from "pinia";
import { reactive, ref } from "vue";
import { useProjectsStore } from "./projects.js";
import { useUiStore } from "./ui.js";

export const useConfigStore = defineStore("config", () => {
    const projectsStore = useProjectsStore();
    const uiStore = useUiStore();

    const cfgLoading = ref(false);
    const cfgLoaded = ref(false);
    const cfgSuccess = ref(null);
    const cfg = ref({ projects_dir: "", ignore: [], projects: {} });
    const dirDraft = ref("");
    const savingDir = ref(false);
    const newIgnore = ref("");
    const addingIgnore = ref(false);
    const removingIgnore = reactive({});
    const newSubdirProject = ref("");
    const newSubdirList = ref("");
    const editingSubdirs = reactive({});

    async function fetchConfig() {
        cfgLoading.value = true;
        try {
            const res = await fetch("/api/config");
            if (!res.ok) throw new Error(`HTTP ${res.status}`);
            const data = await res.json();
            cfg.value = {
                projects_dir: data.projects_dir ?? "",
                ignore: data.ignore ?? [],
                projects: data.projects ?? {},
            };
            dirDraft.value = cfg.value.projects_dir;
            cfgLoaded.value = true;
        } catch (e) {
            projectsStore.setActionError(`Could not load config: ${e.message}`);
        } finally {
            cfgLoading.value = false;
        }
    }

    function flash(msg) {
        cfgSuccess.value = msg;
        setTimeout(() => {
            cfgSuccess.value = null;
        }, 3000);
    }

    async function saveDir() {
        const path = dirDraft.value.trim();
        if (!path || path === cfg.value.projects_dir) return;
        savingDir.value = true;
        try {
            const res = await fetch("/api/config/dir", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ path }),
            });
            const data = await res.json();
            if (data.ok) {
                cfg.value.projects_dir = path;
                flash("Projects directory saved");
                projectsStore.fetchProjects({ silent: true });
            } else projectsStore.setActionError(data.error);
        } finally {
            savingDir.value = false;
        }
    }

    async function addIgnore() {
        const project = newIgnore.value.trim();
        if (!project) return;
        addingIgnore.value = true;
        try {
            const res = await fetch("/api/config/ignore", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ project }),
            });
            const data = await res.json();
            if (data.ok) {
                cfg.value.ignore = [...cfg.value.ignore, project];
                newIgnore.value = "";
                flash(`"${project}" ignored`);
                projectsStore.fetchProjects({ silent: true });
            } else projectsStore.setActionError(data.error);
        } finally {
            addingIgnore.value = false;
        }
    }

    async function removeIgnore(project) {
        removingIgnore[project] = true;
        try {
            const res = await fetch(
                `/api/config/ignore/${encodeURIComponent(project)}`,
                { method: "DELETE" },
            );
            const data = await res.json();
            if (data.ok) {
                cfg.value.ignore = cfg.value.ignore.filter(
                    (x) => x !== project,
                );
                flash(`"${project}" removed`);
                projectsStore.fetchProjects({ silent: true });
            } else projectsStore.setActionError(data.error);
        } finally {
            delete removingIgnore[project];
        }
    }

    function startEdit(name, subdirs) {
        editingSubdirs[name] = subdirs.join(",");
    }

    function cancelEdit(name) {
        delete editingSubdirs[name];
    }

    function setEditingSubdir(name, value) {
        editingSubdirs[name] = value;
    }

    async function saveSubdirs(name) {
        const subdirs = (editingSubdirs[name] ?? "")
            .split(",")
            .map((s) => s.trim())
            .filter(Boolean);
        const res = await fetch(
            `/api/config/projects/${encodeURIComponent(name)}/subdirs`,
            {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ subdirs }),
            },
        );
        const data = await res.json();
        if (data.ok) {
            cfg.value.projects = { ...cfg.value.projects, [name]: { subdirs } };
            delete editingSubdirs[name];
            flash(`Subdirs saved for "${name}"`);
        } else projectsStore.setActionError(data.error);
    }

    async function clearSubdirs(name) {
        const res = await fetch(
            `/api/config/projects/${encodeURIComponent(name)}/subdirs`,
            { method: "DELETE" },
        );
        const data = await res.json();
        if (data.ok) {
            const p = { ...cfg.value.projects };
            delete p[name];
            cfg.value.projects = p;
            flash(`Subdirs cleared for "${name}"`);
        } else projectsStore.setActionError(data.error);
    }

    async function addSubdirConfig() {
        const name = newSubdirProject.value.trim();
        const subdirs = newSubdirList.value
            .split(",")
            .map((s) => s.trim())
            .filter(Boolean);
        if (!name || !subdirs.length) return;
        const res = await fetch(
            `/api/config/projects/${encodeURIComponent(name)}/subdirs`,
            {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ subdirs }),
            },
        );
        const data = await res.json();
        if (data.ok) {
            cfg.value.projects = { ...cfg.value.projects, [name]: { subdirs } };
            newSubdirProject.value = "";
            newSubdirList.value = "";
            flash(`Subdirs set for "${name}"`);
        } else projectsStore.setActionError(data.error);
    }

    function openConfig() {
        uiStore.showConfig();
        if (!cfgLoaded.value) fetchConfig();
    }

    return {
        cfgLoading,
        cfgLoaded,
        cfgSuccess,
        cfg,
        dirDraft,
        savingDir,
        newIgnore,
        addingIgnore,
        removingIgnore,
        newSubdirProject,
        newSubdirList,
        editingSubdirs,
        fetchConfig,
        saveDir,
        addIgnore,
        removeIgnore,
        startEdit,
        cancelEdit,
        setEditingSubdir,
        saveSubdirs,
        clearSubdirs,
        addSubdirConfig,
        openConfig,
    };
});
