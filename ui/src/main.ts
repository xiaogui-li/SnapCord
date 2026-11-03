import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import SettingsWindow from "./SettingsWindow.vue";
import PinView from "./PinView.vue";
import CaptureView from "./CaptureView.vue";
import EditorView from "./EditorView.vue";
import "./assets/main.css";

const params = new URLSearchParams(window.location.search);
const Root = params.has("editor")
  ? EditorView
  : params.has("pin")
    ? PinView
    : params.has("capture")
      ? CaptureView
      : params.has("settings")
        ? SettingsWindow
        : App;

if (Root === App) {
  // 主窗口是贴在屏幕顶边的透明置物架，桌面要能透出来。
  document.documentElement.classList.add("overlay-window");
}

if (Root === SettingsWindow) {
  // 设置是独立窗口，同样保持透明背景 + 自绘玻璃框。
  document.documentElement.classList.add("settings-window");
}

const app = createApp(Root);
app.use(createPinia());
app.mount("#app");
