import { computed, onUnmounted, watch } from "vue";

export function useProjectShellSession({ getDrawerName, shell }) {
    const {
        shellProject,
        shellService,
        termEl,
        openShell,
        closeShell,
        attachShell,
        fitShell,
    } = shell;

    const shellVisible = computed(
        () => shellService.value && shellProject.value === getDrawerName(),
    );

    function setTermRef(el) {
        termEl.value = el;
    }

    function confirmShellLoss() {
        if (!shellService.value) return true;
        return window.confirm(
            "A terminal session is open. Continuing will close it.",
        );
    }

    async function openProjectShell(project, service) {
        const replacingShell =
            shellService.value &&
            (shellProject.value !== project || shellService.value !== service);

        if (replacingShell && !confirmShellLoss()) return;
        await openShell(project, service);
    }

    function warnBeforeUnload(e) {
        if (!shellService.value) return;
        e.preventDefault();
        e.returnValue = "";
    }

    watch(shellVisible, (visible) => {
        if (visible) attachShell();
    });

    window.addEventListener("beforeunload", warnBeforeUnload);

    onUnmounted(() => {
        window.removeEventListener("beforeunload", warnBeforeUnload);
    });

    return {
        shellProject,
        shellService,
        shellVisible,
        setTermRef,
        fitShell,
        closeShell,
        confirmShellLoss,
        openProjectShell,
    };
}
