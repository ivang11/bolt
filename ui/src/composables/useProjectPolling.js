import { onUnmounted } from "vue";

export function useProjectPolling(fetchProjects, intervalMs = 15000) {
    fetchProjects();
    let pollInterval = setInterval(
        () => fetchProjects({ silent: true }),
        intervalMs,
    );

    function handleVisibilityChange() {
        if (document.hidden) {
            clearInterval(pollInterval);
        } else {
            clearInterval(pollInterval);
            fetchProjects({ silent: true });
            pollInterval = setInterval(
                () => fetchProjects({ silent: true }),
                intervalMs,
            );
        }
    }

    document.addEventListener("visibilitychange", handleVisibilityChange);

    onUnmounted(() => {
        clearInterval(pollInterval);
        document.removeEventListener(
            "visibilitychange",
            handleVisibilityChange,
        );
    });
}
