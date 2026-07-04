import { describe, it, expect, beforeEach, vi } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useSettingsStore } from "../settings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({}),
}));

describe("useSettingsStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("初始默认值", () => {
    const store = useSettingsStore();
    expect(store.settings.theme_mode).toBe("dark");
    expect(store.settings.accent_color).toBe("#f97316");
    expect(store.settings.annotation_line_width).toBe("medium");
    expect(store.settings.show_labels).toBe(true);
    expect(store.settings.status_bar_visible).toBe(true);
    expect(store.settings.dense_mode).toBe(false);
    expect(store.showModal).toBe(false);
  });

  it("openModal / closeModal", () => {
    const store = useSettingsStore();
    store.openModal();
    expect(store.showModal).toBe(true);
    store.closeModal();
    expect(store.showModal).toBe(false);
  });

  it("setAccent 修改强调色", () => {
    const store = useSettingsStore();
    store.setAccent("#ef4444");
    expect(store.settings.accent_color).toBe("#ef4444");
  });

  it("reset 恢复默认", () => {
    const store = useSettingsStore();
    store.settings.annotation_line_width = "thin";
    store.settings.dense_mode = true;
    store.reset();
    expect(store.settings.annotation_line_width).toBe("medium");
    expect(store.settings.dense_mode).toBe(false);
  });
});
