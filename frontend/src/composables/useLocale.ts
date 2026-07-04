import { watch } from "vue";
import { useI18n } from "vue-i18n";

const LOCALE_KEY = "easy-label-locale";

export function useLocale() {
  const { locale } = useI18n();

  function loadLocale() {
    const saved = localStorage.getItem(LOCALE_KEY);
    if (saved === "en-US" || saved === "zh-CN") {
      locale.value = saved;
    }
  }

  function setLocale(l: "zh-CN" | "en-US") {
    locale.value = l;
    localStorage.setItem(LOCALE_KEY, l);
  }

  watch(locale, (val) => {
    localStorage.setItem(LOCALE_KEY, val);
  });

  return { locale, loadLocale, setLocale };
}
