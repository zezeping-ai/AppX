import { createApp } from "vue";
import { createPinia } from "pinia";
import {
  Alert,
  App,
  Button,
  Card,
  Checkbox,
  ConfigProvider,
  Divider,
  Dropdown,
  Input,
  Layout,
  Menu,
  Radio,
  Space,
  Switch,
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

const register = (name: string, component: any) => {
  if (!app.component(name)) {
    app.component(name, component);
  }
};

[
  Alert,
  App,
  Button,
  Card,
  Checkbox,
  ConfigProvider,
  Divider,
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
  Switch,
  Tag,
  Typography,
  Typography.Title,
  Typography.Text,
  Typography.Paragraph,
].forEach((component) => {
  if (component?.name) {
    register(component.name, component);
  }
});

register("ATypographyTitle", Typography.Title);
register("ATypographyText", Typography.Text);
register("ATypographyParagraph", Typography.Paragraph);
register("ARadioButton", Radio.Button);
register("ARadioGroup", Radio.Group);
register("AInputPassword", Input.Password);
register("ATextarea", Input.TextArea);
register("AMenuItem", Menu.Item);
register("ADropdown", Dropdown);
register("ADropdownButton", Dropdown.Button);
register("ALayoutSider", Layout.Sider);
register("ALayoutContent", Layout.Content);
register("AApp", App);

app.mount("#app");
