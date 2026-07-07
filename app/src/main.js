import "./app.css";
import { mount } from "svelte";
import { initTheme } from "./theme.js";
import App from "./App.svelte";

initTheme();

export default mount(App, { target: document.getElementById("app") });
