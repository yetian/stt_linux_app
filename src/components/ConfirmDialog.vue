<script setup lang="ts">
import { useI18n } from "vue-i18n";

const props = defineProps<{
  open: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  danger?: boolean;
}>();

const emit = defineEmits<{ confirm: []; cancel: [] }>();
const { t } = useI18n();
</script>

<template>
  <Teleport to="body">
    <div
      v-if="props.open"
      class="fixed inset-0 z-50 grid place-items-center bg-base-950/70 p-4 backdrop-blur-sm"
      @click.self="emit('cancel')"
    >
      <div class="w-full max-w-sm rounded-2xl border border-base-700 bg-base-900 p-5 shadow-2xl">
        <h3 class="text-base font-semibold text-slate-100">⚠️ {{ props.title }}</h3>
        <p class="mt-2 text-sm leading-relaxed text-slate-400">{{ props.message }}</p>
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
            class="rounded-lg px-3 py-1.5 text-xs font-medium transition-colors"
            :class="
              props.danger
                ? 'bg-rose-500 text-white hover:bg-rose-400'
                : 'bg-accent-500 text-base-950 hover:bg-accent-400'
            "
            @click="emit('confirm')"
          >
            {{ props.confirmLabel ?? t("common.confirm") }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
