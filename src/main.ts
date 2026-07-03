import { createApp } from "vue";
import { createPinia } from "pinia";
import {
  Alert,
  App,
  Button,
  Card,
  ConfigProvider,
  Dropdown,
  Input,
  Layout,
  Menu,
  Radio,
  Space,
  Tag,
  Typography,
} from "ant-design-vue";
import "ant-design-vue/dist/reset.css";
import AppRoot from "@/App.vue";
import { router } from "@/router";
import "@/styles/tailwind.css";
import "@/styles/global.scss";

const app = createApp(AppRoot);

app.use(createPinia());
app.use(router);

[
  Alert,
  App,
  Button,
  Card,
  ConfigProvider,
  Input,
  Input.Password,
  Input.TextArea,
  Layout,
  Layout.Content,
  Layout.Sider,
  Menu,
  Menu.Item,
  Radio,
  Radio.Button,
  Radio.Group,
  Space,
  Tag,
  Typography,
  Typography.Title,
  Typography.Text,
  Typography.Paragraph,
].forEach((component) => {
  if (component?.name) {
    app.component(component.name, component);
  }
});

app.component("ATypographyTitle", Typography.Title);
app.component("ATypographyText", Typography.Text);
app.component("ATypographyParagraph", Typography.Paragraph);
app.component("ARadioButton", Radio.Button);
app.component("ARadioGroup", Radio.Group);
app.component("AInputPassword", Input.Password);
app.component("ATextarea", Input.TextArea);
app.component("AMenuItem", Menu.Item);
app.component("ADropdown", Dropdown);
app.component("ADropdownButton", Dropdown.Button);
app.component("ALayoutSider", Layout.Sider);
app.component("ALayoutContent", Layout.Content);
app.component("AApp", App);

app.mount("#app");
