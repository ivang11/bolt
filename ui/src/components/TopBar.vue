<template>
    <header class="topbar">
        <div class="brand">
            <span class="brand-bolt">⚡</span>
            <span class="brand-name">bolt</span>
        </div>
        <div class="brand-sep"></div>
        <nav class="nav-tabs">
            <button
                class="nav-tab"
                :class="{ active: ui.view === 'projects' }"
                @click="ui.showProjects"
            >
                Projects
            </button>
            <button
                class="nav-tab"
                :class="{ active: ui.view === 'config' }"
                @click="config.openConfig"
            >
                Config
            </button>
        </nav>
        <div v-if="ui.view === 'projects'" class="search-wrap">
            <svg
                class="search-icon"
                width="13"
                height="13"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <circle cx="11" cy="11" r="8" />
                <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            <input
                v-model="projects.searchQuery"
                class="search-in"
                type="text"
                placeholder="Filter projects..."
            />
            <button
                v-if="projects.searchQuery"
                class="search-clear"
                @click="projects.searchQuery = ''"
                title="Clear"
            >
                <svg
                    width="10"
                    height="10"
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
        <div class="topbar-right">
            <button
                v-if="ui.view === 'projects'"
                @click="projects.stopAllProjects"
                class="icon-btn"
                title="Stop all projects"
                :disabled="projects.stoppingAll"
            >
                <svg
                    v-if="!projects.stoppingAll"
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                >
                    <rect x="3" y="3" width="18" height="18" rx="2" />
                </svg>
                <svg
                    v-else
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    class="spinning"
                >
                    <path d="M21 2v6h-6" />
                    <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
                    <path d="M3 22v-6h6" />
                    <path d="M21 12a9 9 0 0 1-15 6.7L3 16" />
                </svg>
            </button>
            <button
                v-if="ui.view === 'projects'"
                @click="projects.fetchProjects"
                class="icon-btn"
                title="Refresh"
            >
                <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    :class="{ spinning: projects.loading }"
                >
                    <path d="M21 2v6h-6" />
                    <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
                    <path d="M3 22v-6h6" />
                    <path d="M21 12a9 9 0 0 1-15 6.7L3 16" />
                </svg>
            </button>
        </div>
    </header>
</template>

<script setup>
import { useConfigStore } from "../stores/config.js";
import { useProjectsStore } from "../stores/projects.js";
import { useUiStore } from "../stores/ui.js";

const ui = useUiStore();
const projects = useProjectsStore();
const config = useConfigStore();
</script>
