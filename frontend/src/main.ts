import { createApp } from "vue";
import { createPinia } from "pinia";
import naive from "naive-ui";
import App from "./App.vue";
import "./assets/main.css";

document.body.style.background = "#1e1e1e";
document.body.style.backgroundImage = "none";

const app = createApp(App);
app.use(createPinia());
app.use(naive);
app.mount("#app");
