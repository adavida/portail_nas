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

test("suggestions filter existing groups not in draft and add on click", async () => {
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

  expect(screen.queryByTestId("groups-suggest-alice-devs")).toBeNull();
  expect(screen.queryByTestId("groups-suggest-alice-ops")).toBeNull();

  await fireEvent.input(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "ops" },
  });

  expect(screen.getByTestId("groups-suggest-alice-ops")).toBeInTheDocument();
  expect(
    screen.getByTestId("groups-suggest-alice-opsmaint"),
  ).toBeInTheDocument();

  await fireEvent.click(screen.getByTestId("groups-suggest-alice-ops"));

  expect(screen.getByTestId("groups-chip-alice-ops")).toBeInTheDocument();
});

test("suggestions filter case-insensitively and prefix first", async () => {
  const onSave = vi.fn(async () => true);

  render(GroupsEditor, {
    props: {
      uid: "alice",
      groups: [],
      allGroups: ["devs", "users", "dvops"],
      onSave,
      onCancel: () => {},
    },
  });

  await fireEvent.input(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "DE" },
  });

  expect(screen.queryByTestId("groups-suggest-alice-users")).toBeNull();

  await fireEvent.click(screen.getByTestId("groups-suggest-alice-devs"));

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

  expect(screen.queryByTestId("groups-suggest-alice-user")).toBeNull();
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

  await fireEvent.input(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "devs" },
  });
  await fireEvent.click(screen.getByTestId("groups-suggest-alice-devs"));
  await fireEvent.click(screen.getByTestId("groups-save-alice"));

  await waitFor(() => {
    expect(onSave).toHaveBeenCalledWith(["devs"]);
    expect(onCancel).toHaveBeenCalled();
  });
});
