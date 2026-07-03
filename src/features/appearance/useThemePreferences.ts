import { computed, watchEffect } from "vue";
import { usePreferredDark, useStorage } from "@vueuse/core";

export type ThemeMode = "system" | "light" | "dark";

type AppPreferences = {
  theme: ThemeMode;
};

const DEFAULT_PREFERENCES: AppPreferences = {
  theme: "system",
};

const preferences = useStorage<AppPreferences>(
  "appx.preferences",
  DEFAULT_PREFERENCES,
  localStorage,
  {
    mergeDefaults: true,
  },
);

export function useThemePreferences() {
  const preferredDark = usePreferredDark();

  const resolvedTheme = computed<"light" | "dark">(() => {
    if (preferences.value.theme === "system") {
      return preferredDark.value ? "dark" : "light";
    }
    return preferences.value.theme;
  });

  watchEffect(() => {
    document.documentElement.dataset.theme = resolvedTheme.value;
  });

  const themeMode = computed<ThemeMode>({
    get: () => preferences.value.theme,
    set: (value) => {
      preferences.value = {
        ...preferences.value,
        theme: value,
      };
    },
  });

  return {
    preferences,
    resolvedTheme,
    isDark: computed(() => resolvedTheme.value === "dark"),
    themeMode,
  };
}
