import { onUnmounted } from "vue";

export function useEscapeKey(handler) {
    function handleKeydown(e) {
        if (e.key === "Escape") handler(e);
    }

    document.addEventListener("keydown", handleKeydown);

    onUnmounted(() => {
        document.removeEventListener("keydown", handleKeydown);
    });
}
