<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

const props = defineProps<{
  open: boolean;
  title: string;
  label: string;
  initialValue?: string;
  confirmLabel?: string;
}>();

const emit = defineEmits<{ confirm: [value: string]; cancel: [] }>();
const { t } = useI18n();

const value = ref("");
const input = ref<HTMLInputElement | null>(null);

watch(
  () => props.open,
  async (open) => {
    if (!open) return;
    value.value = props.initialValue ?? "";
    await nextTick();
    input.value?.focus();
    input.value?.select();
  },
  { immediate: true },
);

function submit(): void {
  const trimmed = value.value.trim();
  if (trimmed === "") return;
  emit("confirm", trimmed);
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="props.open"
      class="fixed inset-0 z-50 grid place-items-center bg-base-950/70 p-4 backdrop-blur-sm"
      @click.self="emit('cancel')"
    >
      <div class="w-full max-w-sm rounded-2xl border border-base-700 bg-base-900 p-5 shadow-2xl">
        <h3 class="text-base font-semibold text-slate-100">✏️ {{ props.title }}</h3>
        <label class="mt-4 flex flex-col gap-1">
          <span class="text-[11px] font-medium tracking-wide text-base-500 uppercase">
            {{ props.label }}
          </span>
          <input
            ref="input"
            v-model="value"
            type="text"
            class="rounded-lg border border-accent-500/50 bg-base-850 px-3 py-2 text-sm text-slate-100 outline-none"
            @keydown.enter.prevent="submit"
            @keydown.esc.prevent="emit('cancel')"
          />
        </label>
        <div class="mt-5 flex justify-end gap-2">
          <button
            type="button"
            class="rounded-lg border border-base-700 bg-base-850 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:border-base-600 hover:text-slate-100"
            @click="emit('cancel')"
          >
            {{ t("common.cancel") }}
          </button>
          <button
            type="button"
            class="rounded-lg bg-accent-500 px-3 py-1.5 text-xs font-medium text-base-950 transition-colors hover:bg-accent-400"
            @click="submit"
          >
            {{ props.confirmLabel ?? t("common.save") }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
