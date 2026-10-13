<script setup lang="ts">
import { ref } from "vue";

const props = withDefaults(
  defineProps<{ title: string; icon: string; defaultOpen?: boolean }>(),
  { defaultOpen: true },
);

const open = ref(props.defaultOpen);
</script>

<template>
  <section class="flex flex-col">
    <div class="flex items-center justify-between gap-1 px-1">
      <button
        type="button"
        class="flex flex-1 items-center gap-1.5 py-1 text-[11px] font-medium tracking-wide text-base-500 uppercase transition-colors hover:text-slate-300"
        @click="open = !open"
      >
        <i :class="['fa-solid', icon]"></i>
        <span>{{ title }}</span>
        <i
          class="fa-solid text-[8px]"
          :class="open ? 'fa-chevron-down' : 'fa-chevron-right'"
        ></i>
      </button>
      <slot name="action" />
    </div>

    <div v-show="open" class="mt-0.5 flex flex-col gap-0.5">
      <slot />
    </div>
  </section>
</template>
