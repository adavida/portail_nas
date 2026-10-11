import { render } from "@testing-library/svelte";
import type { Component } from "svelte";
import Toaster from "../components/Toaster.svelte";

export function renderWithToaster<P extends Record<string, unknown>>(
  component: Component<P>,
  props: P,
) {
  render(Toaster);
  return render(component, props);
}
