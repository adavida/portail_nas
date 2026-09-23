import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { GroupsEditor } from "./GroupsEditor";

test("chips render draft groups and x removes", () => {
  const onSave = vi.fn(async () => true);

  render(
    <GroupsEditor
      uid="alice"
      groups={["devs"]}
      allGroups={["devs", "ops"]}
      onSave={onSave}
      onCancel={() => {}}
    />,
  );

  const chip = screen.getByTestId("groups-chip-alice-devs");

  expect(chip).toHaveTextContent("devs");

  fireEvent.click(screen.getByTestId("groups-remove-alice-devs"));

  expect(chip).not.toBeInTheDocument();
});

test("suggestions filter existing groups not in draft and add on click", () => {
  const onSave = vi.fn(async () => true);

  render(
    <GroupsEditor
      uid="alice"
      groups={["devs"]}
      allGroups={["devs", "ops", "opsmaint"]}
      onSave={onSave}
      onCancel={() => {}}
    />,
  );

  expect(screen.queryByTestId("groups-suggest-alice-devs")).toBeNull();
  expect(screen.queryByTestId("groups-suggest-alice-ops")).toBeNull();

  fireEvent.change(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "ops" },
  });

  expect(screen.getByTestId("groups-suggest-alice-ops")).toBeInTheDocument();
  expect(
    screen.getByTestId("groups-suggest-alice-opsmaint"),
  ).toBeInTheDocument();

  fireEvent.click(screen.getByTestId("groups-suggest-alice-ops"));

  expect(screen.getByTestId("groups-chip-alice-ops")).toBeInTheDocument();
});

test("suggestions filter case-insensitively and prefix first", () => {
  const onSave = vi.fn(async () => true);

  render(
    <GroupsEditor
      uid="alice"
      groups={[]}
      allGroups={["devs", "users", "dvops"]}
      onSave={onSave}
      onCancel={() => {}}
    />,
  );

  fireEvent.change(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "DE" },
  });

  expect(screen.queryByTestId("groups-suggest-alice-users")).toBeNull();

  fireEvent.click(screen.getByTestId("groups-suggest-alice-devs"));

  expect(screen.getByTestId("groups-chip-alice-devs")).toBeInTheDocument();
});

test("enter with existing gid adds without create", () => {
  const onSave = vi.fn(async () => true);
  const onCreateGroup = vi.fn();

  render(
    <GroupsEditor
      uid="alice"
      groups={[]}
      allGroups={["devs"]}
      onSave={onSave}
      onCreateGroup={onCreateGroup}
      onCancel={() => {}}
    />,
  );

  fireEvent.change(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "DEVS" },
  });
  fireEvent.keyDown(screen.getByTestId("groups-new-input-alice"), {
    key: "Enter",
  });

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

  render(
    <GroupsEditor
      uid="alice"
      groups={[]}
      allGroups={[]}
      onSave={onSave}
      onCreateGroup={onCreateGroup}
      onCancel={() => {}}
    />,
  );

  fireEvent.change(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "devs" },
  });
  fireEvent.keyDown(screen.getByTestId("groups-new-input-alice"), {
    key: "Enter",
  });

  expect(onCreateGroup).toHaveBeenCalledWith("devs");
  expect(screen.queryByTestId("groups-chip-alice-devs")).toBeNull();

  resolve({ ok: true });

  expect(
    await screen.findByTestId("groups-chip-alice-devs"),
  ).toBeInTheDocument();
});

test("save passes draft and cancel signals parent", async () => {
  const onSave = vi.fn(async (next: string[]) => next.length === 1);
  const onCancel = vi.fn();

  render(
    <GroupsEditor
      uid="alice"
      groups={["devs"]}
      allGroups={["devs"]}
      onSave={onSave}
      onCancel={onCancel}
    />,
  );

  fireEvent.click(screen.getByTestId("groups-remove-alice-devs"));
  fireEvent.click(screen.getByTestId("groups-save-alice"));

  await waitFor(() => {
    expect(onSave).toHaveBeenCalledWith([]);
  });
  expect(onCancel).not.toHaveBeenCalled();

  fireEvent.change(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "devs" },
  });
  fireEvent.click(screen.getByTestId("groups-suggest-alice-devs"));
  fireEvent.click(screen.getByTestId("groups-save-alice"));

  await waitFor(() => {
    expect(onSave).toHaveBeenCalledWith(["devs"]);
    expect(onCancel).toHaveBeenCalled();
  });
});
