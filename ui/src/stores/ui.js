import { defineStore } from "pinia";
import { ref } from "vue";

export const useUiStore = defineStore("ui", () => {
    const view = ref("projects");
    const actionError = ref(null);

    function setActionError(message) {
        actionError.value = message;
    }

    function dismissActionError() {
        actionError.value = null;
    }

    function showProjects() {
        view.value = "projects";
    }

    function showConfig() {
        view.value = "config";
    }

    return {
        view,
        actionError,
        setActionError,
        dismissActionError,
        showProjects,
        showConfig,
    };
});
