import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import Sidebar from "./Sidebar.svelte";

test("admin sees all tabs, user only home", () => {
  const { unmount } = render(Sidebar, { props: { admin: true } });

  expect(screen.getByTestId("tab-debug")).toBeInTheDocument();

  unmount();
  render(Sidebar, { props: { admin: false } });

  expect(screen.queryByTestId("tab-users")).not.toBeInTheDocument();
  expect(screen.getByTestId("tab-home")).toBeInTheDocument();
});

test("burger toggles open class", async () => {
  render(Sidebar, { props: { admin: false } });

  await fireEvent.click(screen.getByTestId("nav-burger"));

  const aside = screen.getByTestId("nav-burger").closest("aside");

  expect(aside).toHaveClass("open");
});
