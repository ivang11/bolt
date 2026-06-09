<template>
    <div v-show="visible" class="scroll-area">
        <div v-if="cfgLoading" class="empty-state">
            <div class="spin-ring"></div>
            <span>Loading configuration...</span>
        </div>
        <div v-else class="cfg-body">
            <transition name="slide-down">
                <div v-if="cfgSuccess" class="banner success-banner">
                    <svg
                        width="13"
                        height="13"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2.5"
                        stroke-linecap="round"
                    >
                        <polyline points="20 6 9 17 4 12" />
                    </svg>
                    <span>{{ cfgSuccess }}</span>
                </div>
            </transition>

            <section class="cfg-section">
                <div class="cfg-section-head">
                    <div class="cfg-icon">
                        <svg
                            width="13"
                            height="13"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                        >
                            <path
                                d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                            />
                        </svg>
                    </div>
                    <div>
                        <h2 class="cfg-title">Projects Directory</h2>
                        <p class="cfg-desc">
                            Root folder where Bolt discovers Docker compose
                            projects
                        </p>
                    </div>
                </div>
                <div class="field-row">
                    <input
                        v-model="dirDraft"
                        class="text-in mono"
                        type="text"
                        placeholder="/home/user/projects"
                        @keydown.enter="config.saveDir"
                    />
                    <button
                        class="primary-btn"
                        :disabled="
                            savingDir ||
                            !dirDraft.trim() ||
                            dirDraft === cfg.projects_dir
                        "
                        @click="config.saveDir"
                    >
                        {{ savingDir ? "Saving..." : "Save" }}
                    </button>
                </div>
            </section>

            <div class="cfg-divider"></div>

            <section class="cfg-section">
                <div class="cfg-section-head">
                    <div class="cfg-icon">
                        <svg
                            width="13"
                            height="13"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                        >
                            <circle cx="12" cy="12" r="10" />
                            <line x1="4.93" y1="4.93" x2="19.07" y2="19.07" />
                        </svg>
                    </div>
                    <div>
                        <h2 class="cfg-title">Ignored Projects</h2>
                        <p class="cfg-desc">
                            These projects are hidden from list and switch
                            commands
                        </p>
                    </div>
                </div>
                <div class="chips-wrap">
                    <span v-if="cfg.ignore.length === 0" class="no-items"
                        >No ignored projects</span
                    >
                    <span
                        v-for="project in cfg.ignore"
                        :key="project"
                        class="chip"
                    >
                        {{ project }}
                        <button
                            class="chip-x"
                            :disabled="removingIgnore[project]"
                            @click="config.removeIgnore(project)"
                        >
                            <svg
                                width="9"
                                height="9"
                                viewBox="0 0 12 12"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2.5"
                                stroke-linecap="round"
                            >
                                <line x1="1" y1="1" x2="11" y2="11" />
                                <line x1="11" y1="1" x2="1" y2="11" />
                            </svg>
                        </button>
                    </span>
                </div>
                <div class="field-row" style="margin-top: 12px">
                    <input
                        v-model="newIgnore"
                        class="text-in"
                        type="text"
                        placeholder="project-name"
                        @keydown.enter="config.addIgnore"
                    />
                    <button
                        class="primary-btn"
                        :disabled="addingIgnore || !newIgnore.trim()"
                        @click="config.addIgnore"
                    >
                        {{ addingIgnore ? "Adding..." : "Add" }}
                    </button>
                </div>
            </section>

            <div class="cfg-divider"></div>

            <section class="cfg-section">
                <div class="cfg-section-head">
                    <div class="cfg-icon">
                        <svg
                            width="13"
                            height="13"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                        >
                            <polyline
                                points="22 12 18 12 15 21 9 3 6 12 2 12"
                            />
                        </svg>
                    </div>
                    <div>
                        <h2 class="cfg-title">Project Subdirs</h2>
                        <p class="cfg-desc">
                            Restrict which subdirectories are managed per
                            project
                        </p>
                    </div>
                </div>
                <div class="subdirs-table">
                    <div class="tbl-head">
                        <span>Project</span><span>Subdirectories</span
                        ><span></span>
                    </div>
                    <div
                        v-if="Object.keys(cfg.projects).length === 0"
                        class="tbl-empty"
                    >
                        No subdir configuration - add one below
                    </div>
                    <div
                        v-for="(projectConfig, name) in cfg.projects"
                        :key="name"
                        class="tbl-row"
                    >
                        <span class="tbl-proj">{{ name }}</span>
                        <div class="tbl-subdirs">
                            <template v-if="editingSubdirs[name] !== undefined">
                                <input
                                    class="text-in small mono"
                                    v-model="editingSubdirs[name]"
                                    :placeholder="
                                        projectConfig.subdirs.join(',')
                                    "
                                    @keydown.enter="config.saveSubdirs(name)"
                                />
                            </template>
                            <template v-else>
                                <span
                                    v-for="subdir in projectConfig.subdirs"
                                    :key="subdir"
                                    class="subdir-pill"
                                    >{{ subdir }}</span
                                >
                            </template>
                        </div>
                        <div class="tbl-actions">
                            <template v-if="editingSubdirs[name] !== undefined">
                                <button
                                    class="row-btn accent"
                                    @click="config.saveSubdirs(name)"
                                >
                                    Save
                                </button>
                                <button
                                    class="row-btn muted"
                                    @click="config.cancelEdit(name)"
                                >
                                    Cancel
                                </button>
                            </template>
                            <template v-else>
                                <button
                                    class="row-btn muted"
                                    @click="
                                        config.startEdit(
                                            name,
                                            projectConfig.subdirs,
                                        )
                                    "
                                >
                                    Edit
                                </button>
                                <button
                                    class="row-btn danger"
                                    @click="config.clearSubdirs(name)"
                                >
                                    Clear
                                </button>
                            </template>
                        </div>
                    </div>
                    <div class="tbl-row tbl-add">
                        <input
                            class="text-in small mono"
                            v-model="newSubdirProject"
                            placeholder="project-name"
                        />
                        <input
                            class="text-in small mono"
                            v-model="newSubdirList"
                            placeholder="api,worker,db"
                            @keydown.enter="config.addSubdirConfig"
                        />
                        <div class="tbl-actions">
                            <button
                                class="row-btn accent"
                                :disabled="
                                    !newSubdirProject.trim() ||
                                    !newSubdirList.trim()
                                "
                                @click="config.addSubdirConfig"
                            >
                                Add
                            </button>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    </div>
</template>

<script setup>
import { storeToRefs } from "pinia";
import { useConfigStore } from "../stores/config.js";

defineProps({
    visible: {
        type: Boolean,
        required: true,
    },
});

const config = useConfigStore();
const {
    cfgLoading,
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
} = storeToRefs(config);
</script>
