import { addMember, createGroup, removeMember } from "../api/groups";
import { translate } from "../i18n";
import { toast } from "./toast";

export async function saveGroups(
  uid: string,
  currentGroups: string[],
  next: string[],
  onUpdated?: () => void,
): Promise<boolean> {
  const adds = next.filter((gid) => !currentGroups.includes(gid));
  const removes = currentGroups.filter((gid) => !next.includes(gid));
  try {
    for (const gid of adds) await addMember(gid, uid);
    for (const gid of removes) await removeMember(gid, uid);
  } catch (e) {
    toast((e as Error).message, true);
    return false;
  }
  toast(translate("common.saved"));
  onUpdated?.();
  return true;
}

export async function createGroupWithMember(
  uid: string,
  gid: string,
  onUpdated?: () => void,
): Promise<{ ok: boolean; error?: string }> {
  try {
    await createGroup({
      gid,
      name: gid,
      description: "",
      members: [uid],
    });
  } catch (e) {
    const error = (e as Error).message;
    toast(error, true);
    return { ok: false, error };
  }
  toast(translate("groups.created", { gid }));
  onUpdated?.();
  return { ok: true };
}
