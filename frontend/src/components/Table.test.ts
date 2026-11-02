import { render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import Table from "./Table.svelte";

test("renders the empty row when isEmpty", () => {
  render(Table, {
    head: ["A", "B"],
    empty: "no data",
    emptyTestid: "t-empty",
    isEmpty: true,
  });

  expect(screen.getByTestId("t-empty")).toHaveTextContent("no data");
});

test("renders head and skips the empty row when not empty", () => {
  render(Table, {
    testid: "t-table",
    head: ["Uid", "Name"],
    empty: "no data",
    emptyTestid: "t-empty",
    isEmpty: false,
  });

  const table = screen.getByTestId("t-table");

  expect(table).toHaveTextContent("Uid");
  expect(table).toHaveTextContent("Name");
  expect(screen.queryByTestId("t-empty")).not.toBeInTheDocument();
});

test("renders data rows with null cells as em dash", () => {
  render(Table, {
    testid: "t-data",
    head: ["A", "B"],
    empty: "no data",
    emptyTestid: "t-empty",
    isEmpty: false,
    data: [{ testid: "row-0", cells: ["x", null] }],
  });

  const row = screen.getByTestId("row-0");

  expect(row).toHaveTextContent("x");
  expect(row).toHaveTextContent("—");
});

test("renders nested tag cells and td testid from cell objects", () => {
  render(Table, {
    testid: "t-tags",
    head: ["A", "B"],
    empty: "no data",
    emptyTestid: "t-empty",
    isEmpty: false,
    data: [
      {
        testid: "row-0",
        cells: [
          { text: "devs", testid: "cell-groups-0" },
          [{ text: "openid", cls: "tag" }],
        ],
      },
    ],
  });

  expect(screen.getByTestId("cell-groups-0")).toHaveTextContent("devs");
  expect(screen.getByTestId("row-0")).toHaveTextContent("openid");
});
