import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { expect, test } from "vitest";
import PhoneInput from "./PhoneInput.svelte";

test("splits a stored number into flag, dial and national digits", () => {
  render(PhoneInput, {
    props: { label: "telephone", value: "+33612345678", testId: "pi" },
  });

  expect(screen.getByTestId("pi")).toHaveValue("6 12 34 56 78");
  expect(screen.getByTestId("pi-dial")).toHaveTextContent("+33");
  expect(screen.getByTestId("pi-country")).toHaveTextContent("🇫🇷");
});

test("formats digits typed without the leading zero", async () => {
  render(PhoneInput, {
    props: { label: "telephone", value: "", testId: "pi" },
  });
  const input = screen.getByTestId("pi");

  await fireEvent.input(input, { target: { value: "612345678" } });

  expect(input).toHaveValue("6 12 34 56 78");
});

test("a leading zero typed by habit is dropped from the display", async () => {
  render(PhoneInput, {
    props: { label: "telephone", value: "", testId: "pi" },
  });
  const input = screen.getByTestId("pi");

  await fireEvent.input(input, { target: { value: "0612345678" } });

  expect(input).toHaveValue("6 12 34 56 78");
});

test("the picker lists countries and switching rewrites the number", async () => {
  render(PhoneInput, {
    props: { label: "telephone", value: "+33612345678", testId: "pi" },
  });

  await fireEvent.click(screen.getByTestId("pi-country"));
  const list = screen.getByTestId("pi-list");
  expect(list).toBeInTheDocument();

  await fireEvent.click(within(list).getByRole("button", { name: /Belgium/ }));

  expect(screen.queryByTestId("pi-list")).not.toBeInTheDocument();
  expect(screen.getByTestId("pi-dial")).toHaveTextContent("+32");
  expect(screen.getByTestId("pi")).toHaveValue("612345678");
});

test("the search field filters the country list", async () => {
  render(PhoneInput, {
    props: { label: "telephone", value: "", testId: "pi" },
  });

  await fireEvent.click(screen.getByTestId("pi-country"));
  const list = screen.getByTestId("pi-list");
  await fireEvent.input(within(list).getByTestId("pi-search"), {
    target: { value: "+32" },
  });

  expect(within(list).getByRole("button", { name: /Belgium/ })).toBeVisible();
  expect(
    within(list).queryByRole("button", { name: /Germany/ }),
  ).not.toBeInTheDocument();
});

test("backspace on an empty number removes the dial code", async () => {
  render(PhoneInput, {
    props: { label: "telephone", value: "+33612345678", testId: "pi" },
  });
  const input = screen.getByTestId("pi");

  await fireEvent.input(input, { target: { value: "" } });
  await fireEvent.keyDown(input, { key: "Backspace" });

  expect(screen.queryByTestId("pi-dial")).not.toBeInTheDocument();
  expect(screen.getByTestId("pi-country")).toHaveTextContent("🌐");
  expect(input).toHaveValue("");
});

test("typing an unambiguous dial code picks the country and moves to the number", async () => {
  render(PhoneInput, {
    props: { label: "telephone", value: "+33612345678", testId: "pi" },
  });
  const input = screen.getByTestId("pi");

  await fireEvent.input(input, { target: { value: "" } });
  await fireEvent.keyDown(input, { key: "Backspace" });
  await fireEvent.input(input, { target: { value: "+33" } });

  expect(screen.getByTestId("pi-dial")).toHaveTextContent("+33");
  expect(input).toHaveValue("");

  await fireEvent.input(input, { target: { value: "612345678" } });

  expect(input).toHaveValue("6 12 34 56 78");
});

test("the defaultCountry prop sets the initial dial and flag", () => {
  render(PhoneInput, {
    props: {
      label: "telephone",
      value: "",
      testId: "pi",
      defaultCountry: "BE",
    },
  });

  expect(screen.getByTestId("pi-dial")).toHaveTextContent("+32");
  expect(screen.getByTestId("pi-country")).toHaveTextContent("🇧🇪");
});
