import { mount } from "svelte";
import App from "./App.svelte";
import { installAuthFetch } from "./auth/oidc";
import "./index.scss";

installAuthFetch();

mount(App, { target: document.getElementById("root")! });
