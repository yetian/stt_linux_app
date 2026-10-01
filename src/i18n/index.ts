import { createI18n } from "vue-i18n";
import en from "./locales/en.json";
import zhCN from "./locales/zh-CN.json";
import de from "./locales/de.json";

export const SUPPORTED_UI_LOCALES = ["en", "zh-CN", "de"] as const;
export type UiLocale = (typeof SUPPORTED_UI_LOCALES)[number];

const STORAGE_KEY = "lra.uiLocale";

export function isUiLocale(value: string): value is UiLocale {
  return (SUPPORTED_UI_LOCALES as readonly string[]).includes(value);
}

function detectLocale(): UiLocale {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && isUiLocale(stored)) {
    return stored;
  }

  const nav = navigator.language;
  if (nav.startsWith("zh")) return "zh-CN";
  if (nav.startsWith("de")) return "de";
  return "en";
}

export const i18n = createI18n({
  legacy: false,
  locale: detectLocale(),
  fallbackLocale: "en",
  messages: {
    en,
    "zh-CN": zhCN,
    de,
  },
});

export function setUiLocale(locale: UiLocale): void {
  i18n.global.locale.value = locale;
  localStorage.setItem(STORAGE_KEY, locale);
  document.documentElement.lang = locale;
}

document.documentElement.lang = i18n.global.locale.value;
