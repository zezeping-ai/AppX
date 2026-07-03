import { createApp } from "vue";
import { createPinia } from "pinia";
import { Button, ConfigProvider, Layout, Space, Typography } from "ant-design-vue";
import "ant-design-vue/dist/reset.css";
import App from "@/App.vue";
import { router } from "@/router";
import "@/styles/tailwind.css";
import "@/styles/global.scss";

const app = createApp(App);

app.use(createPinia());
app.use(router);

[Button, ConfigProvider, Layout, Layout.Content, Space, Typography, Typography.Title, Typography.Text].forEach(
  (component) => {
    if (component?.name) {
      app.component(component.name, component);
    }
  },
);

app.component("ATypographyTitle", Typography.Title);
app.component("ATypographyText", Typography.Text);

app.mount("#app");
