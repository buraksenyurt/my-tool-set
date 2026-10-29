import { mount } from "svelte";
import App from "./App.svelte";
import { theme } from "./lib/theme.svelte";
import "./app.css";

// Apply the saved theme before the first paint, so a dark-mode user never
// sees a white flash.
theme.apply();

const app = mount(App, { target: document.getElementById("app")! });

export default app;
