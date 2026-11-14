import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import { mockFetchByUrl } from "../test/mock_fetch_by_url";
import Directory from "./Directory.svelte";

test("shows users then switches to groups", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/users": [
      { uid: "alice", name: "Alice", email: "a@ex.com", groups: [] },
    ],
    "/api/groups": [{ gid: "devs", name: "Devs", description: "" }],
  });

  render(Directory);

  expect(await screen.findByTestId("users-table")).toBeInTheDocument();

  await fireEvent.click(screen.getByTestId("directory-tab-groups"));

  expect(await screen.findByTestId("groups-table")).toBeInTheDocument();
});
