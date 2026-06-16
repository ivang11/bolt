import { defineStore } from "pinia";
import { nextTick, ref } from "vue";
import { apiSegment } from "../composables/api.js";

export const useBuildLogsStore = defineStore("buildLogs", () => {
    const building = ref({});
    const buildLogs = ref([]);
    const buildRef = ref(null);
    let buildSource = null;

    function setBuildRef(el) {
        buildRef.value = el;
    }

    function clearBuildLogs() {
        buildLogs.value = [];
    }

    function closeBuildSource() {
        if (!buildSource) return;
        buildSource.close();
        buildSource = null;
    }

    function doBuild(name, setActionError = () => {}) {
        closeBuildSource();
        buildLogs.value = [];
        building.value = { ...building.value, [name]: true };
        setActionError(null);

        buildSource = new EventSource(
            `/api/projects/${apiSegment(name)}/build`,
        );
        buildSource.onmessage = async (e) => {
            buildLogs.value = [...buildLogs.value, e.data];
            await nextTick();
            if (buildRef.value) {
                buildRef.value.scrollTop = buildRef.value.scrollHeight;
            }
        };
        buildSource.addEventListener("done", (e) => {
            buildSource.close();
            buildSource = null;
            building.value = { ...building.value, [name]: false };
            if (e.data === "error") setActionError("Build failed");
        });
        buildSource.onerror = () => {
            buildSource?.close();
            buildSource = null;
            building.value = { ...building.value, [name]: false };
        };
    }

    return {
        building,
        buildLogs,
        setBuildRef,
        clearBuildLogs,
        closeBuildSource,
        doBuild,
    };
});
