import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import GroupsEditor from "./GroupsEditor.svelte";

test("chips render draft groups and x removes", async () => {
  const onSave = vi.fn(async () => true);

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: ["devs"],
      allGroups: ["devs", "ops"],
      onSave,
      onCancel: () => {},
    },
  });

  const chip = screen.getByTestId("groups-chip-alice-devs");

  expect(chip).toHaveTextContent("devs");

  await fireEvent.click(screen.getByTestId("groups-remove-alice-devs"));

  expect(chip).not.toBeInTheDocument();
});

test("typing exact gid shows matches without create and enter adds it", async () => {
  const onSave = vi.fn(async () => true);

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: ["devs"],
      allGroups: ["devs", "ops", "opsmaint"],
      onSave,
      onCancel: () => {},
    },
  });

  const input = screen.getByTestId("groups-new-input-alice");
  await fireEvent.input(input, { target: { value: "ops" } });

  expect(screen.queryByTestId("groups-create-alice")).toBeNull();
  expect(screen.getByTestId("groups-suggest-alice-ops")).toBeInTheDocument();

  await fireEvent.keyDown(input, { key: "Enter" });

  expect(screen.getByTestId("groups-chip-alice-ops")).toBeInTheDocument();
});

test("enter keeps editor open when create fails", async () => {
  const onSave = vi.fn(async () => true);
  const onCreateGroup = vi.fn(async () => ({
    ok: false,
    error: "invalid gid",
  }));

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: [],
      allGroups: [],
      onSave,
      onCreateGroup,
      onCancel: () => {},
    },
  });

  const input = screen.getByTestId(
    "groups-new-input-alice",
  ) as HTMLInputElement;
  await fireEvent.input(input, { target: { value: "Dev" } });
  await fireEvent.keyDown(input, { key: "Enter" });

  await waitFor(() => {
    expect(onCreateGroup).toHaveBeenCalledWith("Dev");
  });
  expect(onSave).not.toHaveBeenCalled();
  expect(input.value).toBe("Dev");
});

test("enter on add suggestion keeps the editor open without saving", async () => {
  const onSave = vi.fn(async () => true);
  const onCancel = vi.fn();

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: [],
      allGroups: ["devs"],
      onSave,
      onCancel,
    },
  });

  const input = screen.getByTestId("groups-new-input-alice");
  await fireEvent.input(input, { target: { value: "devs" } });
  await fireEvent.keyDown(input, { key: "Enter" });

  expect(screen.getByTestId("groups-chip-alice-devs")).toBeInTheDocument();
  expect(onSave).not.toHaveBeenCalled();
  expect(onCancel).not.toHaveBeenCalled();
});

test("arrow down then enter adds the matched group", async () => {
  const onSave = vi.fn(async () => true);
  const onCreateGroup = vi.fn();

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: [],
      allGroups: ["devs", "users", "dvops"],
      onSave,
      onCreateGroup,
      onCancel: () => {},
    },
  });

  const input = screen.getByTestId("groups-new-input-alice");
  await fireEvent.input(input, { target: { value: "DE" } });
  await fireEvent.keyDown(input, { key: "ArrowDown" });
  await fireEvent.keyDown(input, { key: "Enter" });

  expect(onCreateGroup).not.toHaveBeenCalled();
  expect(screen.getByTestId("groups-chip-alice-devs")).toBeInTheDocument();
});

test("enter with existing gid adds without create", async () => {
  const onSave = vi.fn(async () => true);
  const onCreateGroup = vi.fn();

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: [],
      allGroups: ["devs"],
      onSave,
      onCreateGroup,
      onCancel: () => {},
    },
  });

  const input = screen.getByTestId("groups-new-input-alice");
  await fireEvent.input(input, { target: { value: "DEVS" } });
  await fireEvent.keyDown(input, { key: "Enter" });

  expect(onCreateGroup).not.toHaveBeenCalled();
  expect(screen.getByTestId("groups-chip-alice-devs")).toBeInTheDocument();
});

test("enter on empty input saves and closes", async () => {
  const onSave = vi.fn(async () => true);
  const onCancel = vi.fn();

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: ["devs"],
      allGroups: ["devs"],
      onSave,
      onCancel,
    },
  });

  await fireEvent.keyDown(screen.getByTestId("groups-new-input-alice"), {
    key: "Enter",
  });

  await waitFor(() => {
    expect(onSave).toHaveBeenCalledWith(["devs"]);
    expect(onCancel).toHaveBeenCalled();
  });
});

test("enter with unknown gid creates group then adds", async () => {
  const onSave = vi.fn(async () => true);
  let resolve: (v: { ok: boolean }) => void = () => {};
  const onCreateGroup = vi.fn(
    () =>
      new Promise<{ ok: boolean }>((r) => {
        resolve = r;
      }),
  );

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: [],
      allGroups: [],
      onSave,
      onCreateGroup,
      onCancel: () => {},
    },
  });

  const input = screen.getByTestId("groups-new-input-alice");
  await fireEvent.input(input, { target: { value: "devs" } });
  await fireEvent.keyDown(input, { key: "Enter" });

  expect(onCreateGroup).toHaveBeenCalledWith("devs");
  expect(screen.queryByTestId("groups-chip-alice-devs")).toBeNull();

  resolve({ ok: true });

  expect(
    await screen.findByTestId("groups-chip-alice-devs"),
  ).toBeInTheDocument();
});

test("backspace on empty input removes last chip", async () => {
  const onSave = vi.fn(async () => true);

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: ["devs"],
      allGroups: ["devs", "ops"],
      onSave,
      onCancel: () => {},
    },
  });

  const input = screen.getByTestId("groups-new-input-alice");
  await fireEvent.input(input, { target: { value: "ops" } });
  await fireEvent.keyDown(input, { key: "Enter" });
  await fireEvent.keyDown(input, { key: "Backspace" });

  expect(screen.getByTestId("groups-chip-alice-devs")).toBeInTheDocument();
  expect(screen.queryByTestId("groups-chip-alice-ops")).toBeNull();
});

test("backspace with text in input keeps chips", async () => {
  const onSave = vi.fn(async () => true);

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: ["devs"],
      allGroups: ["devs", "ops"],
      onSave,
      onCancel: () => {},
    },
  });

  const input = screen.getByTestId("groups-new-input-alice");
  await fireEvent.input(input, { target: { value: "op" } });
  await fireEvent.keyDown(input, { key: "Backspace" });

  expect(screen.getByTestId("groups-chip-alice-devs")).toBeInTheDocument();
});

test("user group is hidden from the editor", async () => {
  const onSave = vi.fn(async () => true);

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: ["user"],
      allGroups: ["user"],
      onSave,
      onCancel: () => {},
    },
  });

  expect(screen.queryByTestId("groups-chip-alice-user")).toBeNull();

  await fireEvent.input(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "user" },
  });

  expect(screen.queryByTestId("groups-suggest-list-alice")).toBeNull();
});

test("backspace skips user master chip", async () => {
  const onSave = vi.fn(async () => true);

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: ["devs", "user"],
      allGroups: ["devs", "user"],
      onSave,
      onCancel: () => {},
    },
  });

  const input = screen.getByTestId("groups-new-input-alice");
  await fireEvent.keyDown(input, { key: "Backspace" });
  await fireEvent.click(screen.getByTestId("groups-save-alice"));

  await waitFor(() => {
    expect(onSave).toHaveBeenCalledWith(["user"]);
  });
});

test("no exact match: first option is a muted create, then matches", async () => {
  const onSave = vi.fn(async () => true);

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: [],
      allGroups: ["devel", "devops"],
      onSave,
      onCancel: () => {},
    },
  });

  const input = screen.getByTestId("groups-new-input-alice");
  await fireEvent.input(input, { target: { value: "dev" } });

  const create = screen.getByTestId("groups-create-alice");
  const items = screen
    .getAllByRole("button")
    .filter((el) => el.classList.contains("opt"));

  expect(create).toHaveTextContent("create « dev »");
  expect(create).toHaveClass("muted");
  expect(items[0]).toBe(create);
  expect(items.length).toBe(3);
  expect((input as HTMLInputElement).value).toBe("dev");
});

test("enter without exact match creates the typed group, not a suggestion", async () => {
  const onSave = vi.fn(async () => true);
  const onCreateGroup = vi.fn(async () => ({ ok: true }));

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: [],
      allGroups: ["devel"],
      onSave,
      onCreateGroup,
      onCancel: () => {},
    },
  });

  const input = screen.getByTestId("groups-new-input-alice");
  await fireEvent.input(input, { target: { value: "dev" } });
  await fireEvent.keyDown(input, { key: "Enter" });

  expect(onCreateGroup).toHaveBeenCalledWith("dev");
  expect(
    await screen.findByTestId("groups-chip-alice-dev"),
  ).toBeInTheDocument();
});

test("save passes draft and cancel signals parent", async () => {
  const onSave = vi.fn(async (next: string[]) => next.length === 1);
  const onCancel = vi.fn();

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: ["devs"],
      allGroups: ["devs"],
      onSave,
      onCancel,
    },
  });

  await fireEvent.click(screen.getByTestId("groups-remove-alice-devs"));
  await fireEvent.click(screen.getByTestId("groups-save-alice"));

  await waitFor(() => {
    expect(onSave).toHaveBeenCalledWith([]);
  });
  expect(onCancel).not.toHaveBeenCalled();

  const input = screen.getByTestId("groups-new-input-alice");
  await fireEvent.input(input, { target: { value: "devs" } });
  await fireEvent.keyDown(input, { key: "Enter" });
  await fireEvent.click(screen.getByTestId("groups-save-alice"));

  await waitFor(() => {
    expect(onSave).toHaveBeenCalledWith(["devs"]);
    expect(onCancel).toHaveBeenCalled();
  });
});

test("escape calls onCancel", async () => {
  const onCancel = vi.fn();

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: ["devs"],
      allGroups: [],
      onSave: async () => true,
      onCancel,
    },
  });

  await fireEvent.keyDown(screen.getByTestId("groups-new-input-alice"), {
    key: "Escape",
  });

  expect(onCancel).toHaveBeenCalled();
});

test("hideActions removes the validate and cancel buttons", () => {
  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: ["devs"],
      allGroups: [],
      onSave: async () => true,
      onCancel: () => {},
      hideActions: true,
    },
  });

  expect(screen.queryByTestId("groups-save-alice")).not.toBeInTheDocument();
  expect(screen.queryByTestId("groups-cancel-alice")).not.toBeInTheDocument();
});
