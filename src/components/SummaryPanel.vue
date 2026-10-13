<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { save } from "@tauri-apps/plugin-dialog";
import DOMPurify from "dompurify";
import { marked } from "marked";
import { systemApi } from "@/api/system";

const props = defineProps<{ markdown?: string | null; fileName?: string | null }>();

const { t } = useI18n();

const view = ref<"preview" | "markdown">("preview");
const copied = ref(false);
const downloading = ref(false);

const html = computed(() =>
  props.markdown
    ? DOMPurify.sanitize(marked.parse(props.markdown) as string)
    : "",
);

async function copy(): Promise<void> {
  if (!props.markdown) return;
  try {
    await navigator.clipboard.writeText(props.markdown);
    copied.value = true;
    window.setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch {
    copied.value = false;
  }
}

async function download(): Promise<void> {
  if (!props.markdown) return;
  downloading.value = true;
  try {
    const stem = (props.fileName ?? "summary").replace(/\.[^./\\]+$/, "");
    const now = new Date();
    const pad = (value: number) => String(value).padStart(2, "0");
    const stamp =
      `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}` +
      `-${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;

    const target = await save({
      defaultPath: `${stem}_summary_${stamp}.md`,
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });

    if (!target) return;
    await systemApi.writeTextFile(target, props.markdown);
  } finally {
    downloading.value = false;
  }
}
</script>

<template>
  <div v-if="markdown" class="flex flex-col gap-3">
    <div class="flex items-center justify-between gap-2">
      <div class="flex items-center gap-1 rounded-lg bg-base-850 p-1">
        <button
          type="button"
          class="rounded-md px-3 py-1 text-xs font-medium transition-colors"
          :class="view === 'preview' ? 'bg-base-800 text-slate-100' : 'text-base-500 hover:text-slate-300'"
          @click="view = 'preview'"
        >
          <i class="fa-solid fa-eye mr-1"></i>{{ t("summary.preview") }}
        </button>
        <button
          type="button"
          class="rounded-md px-3 py-1 text-xs font-medium transition-colors"
          :class="view === 'markdown' ? 'bg-base-800 text-slate-100' : 'text-base-500 hover:text-slate-300'"
          @click="view = 'markdown'"
        >
          <i class="fa-solid fa-code mr-1"></i>{{ t("summary.markdown") }}
        </button>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 hover:text-slate-100"
          @click="copy"
        >
          <i v-if="copied" class="fa-solid fa-check mr-1 text-emerald-400"></i>
          <i v-else class="fa-solid fa-copy mr-1"></i>
          {{ copied ? t("summary.copied") : t("summary.copy") }}
        </button>
        <button
          type="button"
          class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-accent-500 hover:text-slate-100 disabled:opacity-40"
          :disabled="downloading"
          @click="download"
        >
          <i class="fa-solid fa-download mr-1"></i>{{ t("summary.download") }}
        </button>
      </div>
    </div>

    <div
      v-if="view === 'preview'"
      class="prose prose-invert prose-sm max-w-none rounded-xl bg-base-850 p-5"
      v-html="html"
    />
    <pre
      v-else
      class="overflow-x-auto rounded-xl bg-base-850 p-5 text-xs leading-relaxed whitespace-pre-wrap text-slate-300"
    >{{ markdown }}</pre>
  </div>

  <div v-else class="grid place-items-center rounded-2xl border border-base-800 bg-base-900 px-6 py-16 text-center">
    <div class="max-w-sm">
      <div class="mx-auto grid size-12 place-items-center rounded-full bg-base-800 text-base-500">
        <i class="fa-solid fa-file-lines"></i>
      </div>
      <p class="mt-4 text-sm font-medium text-slate-300">{{ t("summary.empty") }}</p>
      <p class="mt-1 text-xs text-base-500">{{ t("summary.emptyHint") }}</p>
    </div>
  </div>
</template>
