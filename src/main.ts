import { createApp } from "vue";
import { createPinia } from "pinia";
import {
  Alert,
  App,
  Button,
  Card,
  Checkbox,
  Col,
  ConfigProvider,
  Divider,
  Dropdown,
  Form,
  Input,
  InputNumber,
  Layout,
  Menu,
  Modal,
  Radio,
  Row,
  Select,
  Space,
  Spin,
  Switch,
  Table,
  Tag,
  Tooltip,
  Typography,
} from "ant-design-vue";
import "ant-design-vue/dist/reset.css";
import AppRoot from "@/App.vue";
import { router } from "@/router";
import { setupLocalIcons } from "@/shared/iconify";
import "@/styles/tailwind.css";
import "@/styles/global.scss";

setupLocalIcons();

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
  Col,
  ConfigProvider,
  Divider,
  Form,
  Form.Item,
  Input,
  Input.Password,
  Input.TextArea,
  InputNumber,
  Layout,
  Layout.Content,
  Layout.Sider,
  Menu,
  Menu.Item,
  Modal,
  Radio,
  Radio.Button,
  Radio.Group,
  Row,
  Select,
  Select.Option,
  Space,
  Spin,
  Switch,
  Table,
  Table.Column,
  Tag,
  Tooltip,
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
register("AInputNumber", InputNumber);
register("AMenuItem", Menu.Item);
register("ADropdown", Dropdown);
register("ADropdownButton", Dropdown.Button);
register("ALayoutSider", Layout.Sider);
register("ALayoutContent", Layout.Content);
register("AFormItem", Form.Item);
register("ATableColumn", Table.Column);
register("ASelectOption", Select.Option);
register("ASpin", Spin);
register("ATooltip", Tooltip);
register("AApp", App);

app.mount("#app");
