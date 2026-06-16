<template>
    <div v-if="ports.length > 0" class="dw-section">
        <div class="dw-label">Ports</div>
        <div class="dw-ports">
            <div
                v-for="port in ports"
                :key="
                    (port.host_port || port.url) +
                    port.service +
                    port.container_port
                "
                class="port-row"
            >
                <a
                    :href="port.url || 'http://localhost:' + port.host_port"
                    target="_blank"
                    rel="noopener"
                    class="port-host"
                >
                    {{ port.url || "localhost:" + port.host_port }}
                    <svg
                        width="9"
                        height="9"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path
                            d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
                        />
                        <polyline points="15 3 21 3 21 9" />
                        <line x1="10" y1="14" x2="21" y2="3" />
                    </svg>
                </a>
                <svg
                    width="10"
                    height="10"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    class="port-arrow-icon"
                >
                    <line x1="5" y1="12" x2="19" y2="12" />
                    <polyline points="12 5 19 12 12 19" />
                </svg>
                <span class="port-svc">{{ port.service }}</span>
                <template v-if="port.host_port > 0">
                    <span class="port-container"
                        >:{{ port.container_port }}</span
                    >
                    <span class="port-proto">{{ port.protocol }}</span>
                </template>
            </div>
        </div>
    </div>
</template>

<script setup>
defineProps({
    ports: {
        type: Array,
        required: true,
    },
});
</script>
